// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(mobile)]
    #[error(transparent)]
    PluginInvoke(#[from] tauri::plugin::mobile::PluginInvokeError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("unknown program {0}")]
    UnknownProgramName(String),
    /// At least one argument did not pass input validation.
    #[error("Scoped command argument at position {index} was found, but failed regex validation {validation}")]
    Validation {
        /// Index of the variable.
        index: usize,

        /// Regex that the variable value failed to match.
        validation: String,
    },
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
