//! Interface for building Tauri plugins.

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "build")]
mod build;
#[cfg(feature = "runtime")]
mod runtime;

#[cfg(feature = "build")]
#[cfg_attr(docsrs, doc(feature = "build"))]
pub use build::*;
#[cfg(feature = "runtime")]
#[cfg_attr(docsrs, doc(feature = "runtime"))]
#[allow(unused)]
pub use runtime::*;
