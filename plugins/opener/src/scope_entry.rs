// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(untagged, rename_all = "camelCase")]
#[allow(unused)]
pub(crate) enum EntryRaw {
    Url { url: String },
    Path { path: PathBuf },
}
