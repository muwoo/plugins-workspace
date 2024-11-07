use tauri::{AppHandle, Runtime, State};

use crate::{open::Program, Opener};

#[tauri::command]
pub async fn open<R: Runtime>(
    _app: AppHandle<R>,
    opener: State<'_, Opener<R>>,
    path: String,
    with: Option<Program>,
) -> crate::Result<()> {
    opener.open(path, with)
}

#[tauri::command]
pub async fn reveal_in_dir() {}
