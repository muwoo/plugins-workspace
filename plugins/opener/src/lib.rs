// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use config::OpenScope;
use tauri::{
    plugin::{Builder, TauriPlugin},
    AppHandle, Manager, Runtime,
};

#[cfg(mobile)]
use tauri::plugin::PluginHandle;
#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "app.tauri.opener";
#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_opener);

mod commands;
mod config;
mod error;
mod open;

pub use error::Error;
type Result<T> = std::result::Result<T, Error>;

pub struct Opener<R: Runtime> {
    #[allow(dead_code)]
    app: AppHandle<R>,
    #[cfg(mobile)]
    mobile_plugin_handle: PluginHandle<R>,
    open_scope: OpenScope,
}

impl<R: Runtime> Opener<R> {
    /// Open a (url) path with a default or specific browser opening program.
    ///
    /// See [`crate::open::open`] for how it handles security-related measures.
    #[cfg(desktop)]
    pub fn open(&self, path: impl Into<String>, with: Option<open::Program>) -> Result<()> {
        open::open(&self.open_scope, path.into(), with).map_err(Into::into)
    }

    /// Open a (url) path with a default or specific browser opening program.
    ///
    /// See [`crate::open::open`] for how it handles security-related measures.
    #[cfg(mobile)]
    pub fn open(&self, path: impl Into<String>, _with: Option<open::Program>) -> Result<()> {
        self.mobile_plugin_handle
            .run_mobile_plugin("open", path.into())
            .map_err(Into::into)
    }
}

/// Extensions to [`tauri::App`], [`tauri::AppHandle`], [`tauri::WebviewWindow`], [`tauri::Webview`] and [`tauri::Window`] to access the opener APIs.
pub trait OpenerExt<R: Runtime> {
    fn opener(&self) -> &Opener<R>;
}

impl<R: Runtime, T: Manager<R>> crate::OpenerExt<R> for T {
    fn opener(&self) -> &Opener<R> {
        self.state::<Opener<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R, Option<config::Config>> {
    Builder::<R, Option<config::Config>>::new("opener")
        .js_init_script(include_str!("init-iife.js").to_string())
        .setup(|app, api| {
            let default_config = config::Config::default();
            let config = api.config().as_ref().unwrap_or(&default_config);

            #[cfg(target_os = "android")]
            let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "OpenerPlugin")?;
            #[cfg(target_os = "ios")]
            let handle = api.register_ios_plugin(init_plugin_opener)?;

            app.manage(Opener {
                app: app.clone(),
                open_scope: config.open_scope(),
                #[cfg(mobile)]
                mobile_plugin_handle: handle,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::open,
            commands::reveal_in_dir
        ])
        .build()
}
