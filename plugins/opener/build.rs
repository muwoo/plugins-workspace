// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

#[path = "src/scope_entry.rs"]
#[allow(dead_code)]
mod scope;

/// Opener scope entry.
#[derive(schemars::JsonSchema)]
#[serde(untagged)]
#[allow(unused)]
enum OpenerScopeEntry {
    Url {
        /// A URL that can be opened by the webview when using the Opener APIs.
        ///
        /// Wildcards can be used following the UNIX glob pattern.
        ///
        /// Examples:
        ///
        /// - "https://*" : allows all HTTPS origin
        ///
        /// - "https://*.github.com/tauri-apps/tauri": allows any subdomain of "github.com" with the "tauri-apps/api" path
        ///
        /// - "https://myapi.service.com/users/*": allows access to any URLs that begins with "https://myapi.service.com/users/"
        url: String,
    },
    Path {
        /// A path that can be opened by the webview when using the Opener APIs.
        ///
        /// The pattern can start with a variable that resolves to a system base directory.
        /// The variables are: `$AUDIO`, `$CACHE`, `$CONFIG`, `$DATA`, `$LOCALDATA`, `$DESKTOP`,
        /// `$DOCUMENT`, `$DOWNLOAD`, `$EXE`, `$FONT`, `$HOME`, `$PICTURE`, `$PUBLIC`, `$RUNTIME`,
        /// `$TEMPLATE`, `$VIDEO`, `$RESOURCE`, `$APP`, `$LOG`, `$TEMP`, `$APPCONFIG`, `$APPDATA`,
        /// `$APPLOCALDATA`, `$APPCACHE`, `$APPLOG`.
        path: PathBuf,
    },
}

// Ensure `OpenerScopeEntry` and `scope::EntryRaw` is kept in sync
fn _f() {
    match (scope::EntryRaw::Url { url: String::new() }) {
        scope::EntryRaw::Url { url } => OpenerScopeEntry::Url { url },
        scope::EntryRaw::Path { path } => OpenerScopeEntry::Path { path },
    };
    match (OpenerScopeEntry::Url { url: String::new() }) {
        OpenerScopeEntry::Url { url } => scope::EntryRaw::Url { url },
        OpenerScopeEntry::Path { path } => scope::EntryRaw::Path { path },
    };
}

const COMMANDS: &[&str] = &["open", "reveal_item_in_dir"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .global_api_script_path("./api-iife.js")
        .android_path("android")
        .ios_path("ios")
        .global_scope_schema(schemars::schema_for!(OpenerScopeEntry))
        .build();

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let mobile = target_os == "ios" || target_os == "android";
    alias("desktop", !mobile);
    alias("mobile", mobile);
}

// creates a cfg alias if `has_feature` is true.
// `alias` must be a snake case string.
fn alias(alias: &str, has_feature: bool) {
    println!("cargo:rustc-check-cfg=cfg({alias})");
    if has_feature {
        println!("cargo:rustc-cfg={alias}");
    }
}
