use serde_json::json;
use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use http::header::{HeaderName, HeaderValue};
use serde::{ser::Serializer, Deserialize, Serialize};
use tauri::{
  ipc::Channel,
  plugin::{Builder as PluginBuilder, TauriPlugin},
  Manager, Runtime, State, Window,
};
use tokio::{net::TcpStream, sync::Mutex};
#[cfg(any(feature = "rustls-tls", feature = "native-tls"))]
use tokio_tungstenite::connect_async_tls_with_config;
#[cfg(not(any(feature = "rustls-tls", feature = "native-tls")))]
use tokio_tungstenite::connect_async_with_config;
use tokio_tungstenite::{
  tungstenite::{
    client::IntoClientRequest,
    protocol::{CloseFrame as ProtocolCloseFrame, WebSocketConfig},
    Message,
  },
  Connector, MaybeTlsStream, WebSocketStream,
};

use std::collections::HashMap;
use std::str::FromStr;

type Id = u32;
type WebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;
type WebSocketWriter = SplitSink<WebSocket, Message>;
type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
enum Error {
  #[error(transparent)]
  Websocket(#[from] tokio_tungstenite::tungstenite::Error),
  #[error("connection not found for the given id: {0}")]
  ConnectionNotFound(Id),
  #[error(transparent)]
  InvalidHeaderValue(#[from] tokio_tungstenite::tungstenite::http::header::InvalidHeaderValue),
  #[error(transparent)]
  InvalidHeaderName(#[from] tokio_tungstenite::tungstenite::http::header::InvalidHeaderName),
}

impl Serialize for Error {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
      S: Serializer,
  {
    serializer.serialize_str(self.to_string().as_str())
  }
}

#[derive(Default)]
struct ConnectionManager(Mutex<HashMap<Id, WebSocketWriter>>);

#[cfg(any(feature = "rustls-tls", feature = "native-tls"))]
struct TlsConnector(Mutex<Option<Connector>>);

#[derive(Deserialize)]
#[serde(untagged, rename_all = "camelCase")]
enum Max {
  None,
  Number(usize),
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ConnectionConfig {
  pub read_buffer_size: Option<usize>,
  pub write_buffer_size: Option<usize>,
  pub max_write_buffer_size: Option<usize>,
  pub max_message_size: Option<Max>,
  pub max_frame_size: Option<Max>,
  #[serde(default)]
  pub accept_unmasked_frames: bool,
  pub headers: Option<Vec<(String, String)>>,
}

impl From<ConnectionConfig> for WebSocketConfig {
  fn from(config: ConnectionConfig) -> Self {
    let mut builder =
      WebSocketConfig::default().accept_unmasked_frames(config.accept_unmasked_frames);

    if let Some(read_buffer_size) = config.read_buffer_size {
      builder = builder.read_buffer_size(read_buffer_size)
    }

    if let Some(write_buffer_size) = config.write_buffer_size {
      builder = builder.write_buffer_size(write_buffer_size)
    }

    if let Some(max_write_buffer_size) = config.max_write_buffer_size {
      builder = builder.max_write_buffer_size(max_write_buffer_size)
    }

    if let Some(max_message_size) = config.max_message_size {
      let max_size = match max_message_size {
        Max::None => Option::None,
        Max::Number(n) => Some(n),
      };
      builder = builder.max_message_size(max_size);
    }

    if let Some(max_frame_size) = config.max_frame_size {
      let max_size = match max_frame_size {
        Max::None => Option::None,
        Max::Number(n) => Some(n),
      };
      builder = builder.max_frame_size(max_size);
    }

    builder
  }
}

#[derive(Deserialize, Serialize)]
struct CloseFrame {
  pub code: u16,
  pub reason: String,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "data")]
enum WebSocketMessage {
  Text(String),
  Binary(Vec<u8>),
  Ping(Vec<u8>),
  Pong(Vec<u8>),
  Close(Option<CloseFrame>),
}
use std::sync::Arc;

#[tauri::command]
async fn connect<R: Runtime>(
  window: Window<R>,
  url: String,
  on_message: Channel<serde_json::Value>,
  config: Option<ConnectionConfig>,
) -> Result<Id> {
  println!("connect: {:?}", url);
  let id = rand::random();
  let mut request = url.into_client_request()?;

  if let Some(headers) = config.as_ref().and_then(|c| c.headers.as_ref()) {
    for (k, v) in headers {
      let header_name = HeaderName::from_str(k.as_str())?;
      let header_value = HeaderValue::from_str(v.as_str())?;
      request.headers_mut().insert(header_name, header_value);
    }
  }

  #[cfg(any(feature = "rustls-tls", feature = "native-tls"))]
    let tls_connector = match window.try_state::<TlsConnector>() {
    Some(tls_connector) => tls_connector.0.lock().await.clone(),
    None => None,
  };

  #[cfg(any(feature = "rustls-tls", feature = "native-tls"))]
    let (ws_stream, _) =
    connect_async_tls_with_config(request, config.map(Into::into), false, tls_connector)
      .await?;
  #[cfg(not(any(feature = "rustls-tls", feature = "native-tls")))]
    let (ws_stream, _) = connect_async_with_config(request, config.clone().map(Into::into), false).await?;

  tauri::async_runtime::spawn(async move {
    // 在建立 WebSocket 连接时，通常会将连接分为两个部分
    // 一个用于发送消息（写入部分）
    // 另一个用于接收消息（读取部分）。
    // 这可以通过调用 split() 方法来实现
    let (write, read) = ws_stream.split();
    // 将 write 加入到 ConnectionManager 中
    let manager = window.state::<ConnectionManager>();
    manager.0.lock().await.insert(id, write);

    // 将 window 和 config 转换为 Arc<Mutex<T>> 类型, 以便在异步闭包中使用
    let window_arc = Arc::new(Mutex::new(window));
    let config_arc = Arc::new(Mutex::new(config));

    // read 是一个异步流，使用 for_each 处理方法来处理接收到的消息
    read.for_each(move |message| {
      let window_ = Arc::clone(&window_arc); // 克隆 Arc
      let config_ = Arc::clone(&config_arc); // 克隆 Arc
      let on_message_ = on_message.clone();
      async move {
        if let Ok(Message::Close(_)) = message {
          let manager_lock = window_.lock().await; // 将结果存储在变量中
          let manager = manager_lock.state::<ConnectionManager>();
          manager.0.lock().await.remove(&id);
        }

        let response = match message {
          Ok(Message::Text(t)) => {
            serde_json::to_value(WebSocketMessage::Text(t.to_string())).unwrap()
          }
          Ok(Message::Binary(t)) => {
            serde_json::to_value(WebSocketMessage::Binary(t.to_vec())).unwrap()
          }
          Ok(Message::Ping(t)) => {
            serde_json::to_value(WebSocketMessage::Ping(t.to_vec())).unwrap()
          }
          Ok(Message::Pong(t)) => {
            serde_json::to_value(WebSocketMessage::Pong(t.to_vec())).unwrap()
          }
          Ok(Message::Close(t)) => {
            serde_json::to_value(WebSocketMessage::Close(t.map(|v| CloseFrame {
              code: v.code.into(),
              reason: v.reason.to_string(),
            })))
              .unwrap()
          }
          Ok(Message::Frame(_)) => serde_json::Value::Null, // This value can't be recieved.
          Err(e) => serde_json::to_value(Error::from(e)).unwrap(),
        };

        if let Some(agent_id) = config_.lock().await.as_ref().and_then(|c| c.agent_id.as_ref()) {
          let _ = window_.lock().await.eval(format!("window.frames['{}']?._ws_onmessage({}, {})", &agent_id, &response, id).as_str());
        }
      }
    })
      .await;
  });

  Ok(id)
}

#[tauri::command]
async fn send(
  manager: State<'_, ConnectionManager>,
  id: Id,
  message: WebSocketMessage,
) -> Result<()> {
  if let Some(write) = manager.0.lock().await.get_mut(&id) {
    write
      .send(match message {
        WebSocketMessage::Text(t) => Message::Text(t.into()),
        WebSocketMessage::Binary(t) => Message::Binary(t.into()),
        WebSocketMessage::Ping(t) => Message::Ping(t.into()),
        WebSocketMessage::Pong(t) => Message::Pong(t.into()),
        WebSocketMessage::Close(t) => Message::Close(t.map(|v| ProtocolCloseFrame {
          code: v.code.into(),
          reason: v.reason.into(),
        })),
      })
      .await?;
    Ok(())
  } else {
    Err(Error::ConnectionNotFound(id))
  }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::default().build()
}

#[derive(Default)]
pub struct Builder {
  tls_connector: Option<Connector>,
}

impl Builder {
  pub fn new() -> Self {
    Self {
      tls_connector: None,
    }
  }

  pub fn tls_connector(mut self, connector: Connector) -> Self {
    self.tls_connector.replace(connector);
    self
  }

  pub fn build<R: Runtime>(self) -> TauriPlugin<R> {
    PluginBuilder::new("websocket")
      .invoke_handler(tauri::generate_handler![connect, send])
      .setup(|app, _api| {
        app.manage(ConnectionManager::default());
        #[cfg(any(feature = "rustls-tls", feature = "native-tls"))]
        app.manage(TlsConnector(Mutex::new(self.tls_connector)));
        Ok(())
      })
      .build()
  }
}
