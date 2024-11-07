use std::{marker::PhantomData, path::PathBuf, sync::Arc};

use tauri::{ipc::ScopeObject, utils::acl::Value, AppHandle, Manager, Runtime};

use url::Url;
use urlpattern::UrlPatternMatchInput;

use crate::{scope_entry::EntryRaw, Error};

#[derive(Debug)]
pub enum Entry {
    Url(urlpattern::UrlPattern),
    Path(Option<PathBuf>),
}

fn parse_url_pattern(
    s: &str,
) -> std::result::Result<urlpattern::UrlPattern, urlpattern::quirks::Error> {
    let mut init = urlpattern::UrlPatternInit::parse_constructor_string::<regex::Regex>(s, None)?;
    if init.search.as_ref().map(|p| p.is_empty()).unwrap_or(true) {
        init.search.replace("*".to_string());
    }
    if init.hash.as_ref().map(|p| p.is_empty()).unwrap_or(true) {
        init.hash.replace("*".to_string());
    }
    if init
        .pathname
        .as_ref()
        .map(|p| p.is_empty() || p == "/")
        .unwrap_or(true)
    {
        init.pathname.replace("*".to_string());
    }
    urlpattern::UrlPattern::parse(init, Default::default())
}

impl ScopeObject for Entry {
    type Error = Error;

    fn deserialize<R: Runtime>(
        app: &AppHandle<R>,
        raw: Value,
    ) -> std::result::Result<Self, Self::Error> {
        serde_json::from_value(raw.into())
            .and_then(|raw| {
                let entry = match raw {
                    EntryRaw::Url { url } => Entry::Url(parse_url_pattern(&url).map_err(|e| {
                        serde::de::Error::custom(format!(
                            "`{}` is not a valid URL pattern: {e}",
                            url
                        ))
                    })?),
                    EntryRaw::Path { path } => {
                        let path = match app.path().parse(path) {
                            Ok(path) => Some(path),
                            #[cfg(not(target_os = "android"))]
                            Err(tauri::Error::UnknownPath) => None,
                            Err(err) => return Err(serde::de::Error::custom(err.to_string())),
                        };

                        Entry::Path(path)
                    }
                };

                Ok(entry)
            })
            .map_err(Into::into)
    }
}

#[derive(Debug)]
pub struct Scope<'a, R: Runtime, M: Manager<R>> {
    allowed: Vec<&'a Arc<Entry>>,
    denied: Vec<&'a Arc<Entry>>,
    manager: &'a M,
    _marker: PhantomData<R>,
}

impl<'a, R: Runtime, M: Manager<R>> Scope<'a, R, M> {
    pub(crate) fn new(
        manager: &'a M,
        allowed: Vec<&'a Arc<Entry>>,
        denied: Vec<&'a Arc<Entry>>,
    ) -> Self {
        Self {
            manager,
            allowed,
            denied,
            _marker: PhantomData,
        }
    }

    pub fn is_allowed(&self, path_or_url: &str) -> crate::Result<bool> {
        let url = Url::parse(path_or_url).ok();
        match url {
            Some(url) => Ok(self.is_url_allowed(url)),
            None => self.is_path_allowed(path_or_url),
        }
    }

    pub fn is_url_allowed(&self, url: Url) -> bool {
        let denied = self.denied.iter().any(|entry| match entry.as_ref() {
            Entry::Url(url_pattern) => url_pattern
                .test(UrlPatternMatchInput::Url(url.clone()))
                .unwrap_or_default(),
            Entry::Path { .. } => false,
        });
        if denied {
            false
        } else {
            self.allowed.iter().any(|entry| match entry.as_ref() {
                Entry::Url(url_pattern) => url_pattern
                    .test(UrlPatternMatchInput::Url(url.clone()))
                    .unwrap_or_default(),
                Entry::Path { .. } => false,
            })
        }
    }

    pub fn is_path_allowed(&self, path: &str) -> crate::Result<bool> {
        let fs_scope = tauri::fs::Scope::new(
            self.manager,
            &tauri::utils::config::FsScope::Scope {
                allow: self
                    .allowed
                    .iter()
                    .filter_map(|e| match e.as_ref() {
                        Entry::Path(path) => path.clone(),
                        _ => None,
                    })
                    .collect(),
                deny: self
                    .denied
                    .iter()
                    .filter_map(|e| match e.as_ref() {
                        Entry::Path(path) => path.clone(),
                        _ => None,
                    })
                    .collect(),
                require_literal_leading_dot: None,
            },
        )?;

        Ok(fs_scope.is_allowed(path))
    }
}
