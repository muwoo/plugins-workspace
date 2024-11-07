use regex::Regex;
use serde::Deserialize;

/// Scope for the open command
pub struct OpenScope {
    /// The validation regex that `shell > open` paths must match against.
    pub open: Option<Regex>,
}

/// Configuration for the shell plugin.
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    /// Open URL with the user's default application.
    #[serde(default)]
    pub open: OpenConfig,
}

impl Config {
    pub fn open_scope(&self) -> OpenScope {
        let open = match &self.open {
            OpenConfig::Flag(false) => None,
            OpenConfig::Flag(true) => {
                Some(Regex::new(r"^((mailto:\w+)|(tel:\w+)|(https?://\w+)).+").unwrap())
            }
            OpenConfig::Validate(validator) => {
                let regex = format!("^{validator}$");
                let validator =
                    Regex::new(&regex).unwrap_or_else(|e| panic!("invalid regex {regex}: {e}"));
                Some(validator)
            }
        };

        OpenScope { open }
    }
}

/// Defines the `opener > open` api scope.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
#[non_exhaustive]
pub enum OpenConfig {
    /// If the opener open API should be enabled.
    ///
    /// If enabled, the default validation regex (`^((mailto:\w+)|(tel:\w+)|(https?://\w+)).+`) is used.
    Flag(bool),

    /// Enable the opener open API, with a custom regex that the opened path must match against.
    ///
    /// The regex string is automatically surrounded by `^...$` to match the full string.
    /// For example the `https?://\w+` regex would be registered as `^https?://\w+$`.
    ///
    /// If using a custom regex to support a non-http(s) schema, care should be used to prevent values
    /// that allow flag-like strings to pass validation. e.g. `--enable-debugging`, `-i`, `/R`.
    Validate(String),
}

impl Default for OpenConfig {
    fn default() -> Self {
        Self::Flag(false)
    }
}
