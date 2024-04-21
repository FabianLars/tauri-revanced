use std::{io, num, path};
use thiserror::Error as DeriveError;

/// Errors returned by the bundler.
#[derive(Debug, DeriveError)]
#[non_exhaustive]
pub enum Error {
    /// Error running tauri_utils API.
    #[error("{0}")]
    Resource(#[from] tauri_utils::Error),
    /// Bundler error.
    #[error("{0}")]
    Bundler(#[from] anyhow::Error),
    /// I/O error.
    #[error("`{0}`")]
    Io(#[from] io::Error),
    /// Image error.
    #[error("`{0}`")]
    Image(#[from] image::ImageError),
    /// Error walking directory.
    #[error("`{0}`")]
    Walkdir(#[from] walkdir::Error),
    /// Strip prefix error.
    #[error("`{0}`")]
    Strip(#[from] path::StripPrefixError),
    /// Number parse error.
    #[error("`{0}`")]
    Convert(#[from] num::TryFromIntError),
    /// Zip error.
    #[error("`{0}`")]
    Zip(#[from] zip::result::ZipError),
    /// Hex error.
    #[error("`{0}`")]
    Hex(#[from] hex::FromHexError),
    /// Handlebars template error.
    #[error("`{0}`")]
    HandleBars(#[from] handlebars::RenderError),
    /// JSON error.
    #[error("`{0}`")]
    Json(#[from] serde_json::error::Error),
    /// Regex error.
    #[cfg(any(target_os = "macos", windows))]
    #[error("`{0}`")]
    RegexError(#[from] regex::Error),
    /// Failed to perform HTTP request.
    #[error("`{0}`")]
    Http(#[from] Box<ureq::Error>),
    /// Invalid glob pattern.
    #[cfg(windows)]
    #[error("{0}")]
    GlobPattern(#[from] glob::PatternError),
    /// Failed to use glob pattern.
    #[cfg(windows)]
    #[error("`{0}`")]
    Glob(#[from] glob::GlobError),
    /// Failed to validate downloaded file hash.
    #[error("hash mismatch of downloaded file")]
    Hash,
    /// Unsupported architecture.
    #[error("Architecture Error: `{0}`")]
    Arch(String),
    /// Couldn't find icons.
    #[error(
        "Could not find Icon paths.  Please make sure they exist in the tauri config JSON file"
    )]
    IconPath,
    /// Couldn't find background file.
    #[error("Could not find background file. Make sure it exists in the tauri config JSON file and extension is png/jpg/gif")]
    BackgroundPath,
    /// Error on path util operation.
    #[error("Path Error:`{0}`")]
    PathUtil(String),
    /// Error on shell script.
    #[error("Shell Scripting Error:`{0}`")]
    ShellScript(String),
    /// Generic error.
    #[error("`{0}`")]
    Generic(String),
    /// No bundled project found for the updater.
    #[error("Unable to find a bundled project for the updater")]
    UnableToFindProject,
    /// String is not UTF-8.
    #[error("string is not UTF-8")]
    Utf8(#[from] std::str::Utf8Error),
    /// Windows SignTool not found.
    #[error("SignTool not found")]
    SignToolNotFound,
    /// Failed to open Windows registry.
    #[error("failed to open registry {0}")]
    OpenRegistry(String),
    /// Failed to get registry value.
    #[error("failed to get {0} value on registry")]
    GetRegistryValue(String),
    /// Unsupported OS bitness.
    #[error("unsupported OS bitness")]
    UnsupportedBitness,
    /// Failed to sign application.
    #[error("failed to sign app: {0}")]
    Sign(String),
    /// time error.
    #[cfg(target_os = "macos")]
    #[error("`{0}`")]
    TimeError(#[from] time::error::Error),
    /// Plist error.
    #[cfg(target_os = "macos")]
    #[error(transparent)]
    Plist(#[from] plist::Error),
}

/// Convenient type alias of Result type.
pub type Result<T> = std::result::Result<T, Error>;
