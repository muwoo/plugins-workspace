use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(untagged, rename_all = "camelCase")]
#[allow(unused)]
pub(crate) enum EntryRaw {
    Url { url: String },
    Path { path: PathBuf },
}
