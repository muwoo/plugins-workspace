// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(mobile)]
    #[error(transparent)]
    PluginInvoke(#[from] tauri::plugin::mobile::PluginInvokeError),
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("unknown program {0}")]
    UnknownProgramName(String),
    #[error("Not allowed to open {0}")]
    NotAllowed(String),
    /// API not supported on the current platform
    #[error("API not supported on the current platform")]
    UnsupportedPlatform,
    #[error(transparent)]
    #[cfg(windows)]
    Win32Error(#[from] windows::core::Error),
    /// Path doesn't have a parent.
    #[error("Path doesn't have a parent: {0}")]
    NoParent(PathBuf),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
