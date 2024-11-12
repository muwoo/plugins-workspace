// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
    sync::Arc,
};

use tauri::{ipc::ScopeObject, utils::acl::Value, AppHandle, Manager, Runtime};

use crate::{scope_entry::EntryRaw, Error};

#[derive(Debug)]
pub enum Entry {
    Url(glob::Pattern),
    Path(Option<PathBuf>),
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
                    EntryRaw::Url { url } => Entry::Url(
                        glob::Pattern::new(&url)
                            .map_err(|e| serde::de::Error::custom(e.to_string()))?,
                    ),
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

    pub fn is_url_allowed(&self, url: &str) -> bool {
        let denied = self.denied.iter().any(|entry| match entry.as_ref() {
            Entry::Url(url_pattern) => url_pattern.matches(url),
            Entry::Path { .. } => false,
        });
        if denied {
            false
        } else {
            self.allowed.iter().any(|entry| match entry.as_ref() {
                Entry::Url(url_pattern) => url_pattern.matches(url),
                Entry::Path { .. } => false,
            })
        }
    }

    pub fn is_path_allowed(&self, path: &Path) -> crate::Result<bool> {
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
