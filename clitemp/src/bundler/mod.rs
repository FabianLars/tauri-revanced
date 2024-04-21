//! The Tauri bundler is a tool that generates installers or app bundles for executables.
//! It supports auto updating through [tauri](https://docs.rs/tauri).
//!
//! # Platform support
//! - macOS
//!   - DMG and App bundles
//! - Linux
//!   - Appimage
//! - Windows
//!   - TODO

pub(crate) mod bundle;
pub(crate) mod error;
pub(crate) use error::*;