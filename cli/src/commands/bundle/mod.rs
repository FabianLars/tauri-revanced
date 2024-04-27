use anyhow::Result;
use log::warn;

use crate::Args;

pub mod linux;
pub mod macos;
pub mod windows;

pub fn run(args: Args, bundle_types: Vec<String>) -> Result<()> {
    let all = bundle_types.is_empty();
    let target_arg = !args.target.is_empty();

    let default_linux = !target_arg && cfg!(target_os = "linux");
    let default_macos = !target_arg && cfg!(target_os = "macos");
    let default_windows = !target_arg && cfg!(target_os = "windows");

    let linux = args.target.iter().any(|t| t.contains("linux"));
    let macos = args.target.iter().any(|t| t.contains("darwin"));
    let windows = args.target.iter().any(|t| t.contains("windows"));

    let mut bundle_paths = Vec::new();

    if linux || default_linux {
        #[cfg(target_os = "linux")]
        if all || bundle_types.iter().any(|v| v == "appimage") {
            bundle_paths.push(linux::appimage::bundle()?);
        }
        #[cfg(not(target_os = "linux"))]
        unsupported("linux", "appimage");

        // TODO: Add support for this even though we can't cross-compile tauri apps with webkitgtk.
        #[cfg(target_os = "linux")]
        if all || bundle_types.iter().any(|v| v == "flatpak") {
            bundle_paths.push(linux::flatpak::bundle()?);
        }
        #[cfg(not(target_os = "linux"))]
        unsupported("linux", "flatpak");
    }

    if macos || default_macos {
        if all || bundle_types.iter().any(|v| v == "appbundle") {
            bundle_paths.push(macos::appbundle::bundle()?);
        }

        #[cfg(target_os = "macos")]
        if all || bundle_types.iter().any(|v| v == "dmg") {
            bundle_paths.push(macos::dmg::bundle()?);
        }
        #[cfg(not(target_os = "macos"))]
        unsupported("macos", "dmg");

        #[cfg(target_os = "macos")]
        if all || bundle_types.iter().any(|v| v == "pkg") {
            bundle_paths.push(macos::pkg::bundle()?);
        }
        #[cfg(not(target_os = "macos"))]
        unsupported("macos", "pkg");
    }

    if windows || default_windows {
        if all || bundle_types.iter().any(|v| v == "msi") {
            bundle_paths.push(windows::msi::bundle()?);
        }

        if all || bundle_types.iter().any(|v| v == "msix") {
            bundle_paths.push(windows::msix::bundle()?);
        }

        #[cfg(target_os = "linux")]
        if all || bundle_types.iter().any(|v| v == "wix") {
            bundle_paths.push(windows::wix::bundle()?);
        }
        #[cfg(not(target_os = "windows"))]
        unsupported("windows", "wix");

        if all || bundle_types.iter().any(|v| v == "zip") {
            bundle_paths.push(windows::zip::bundle()?);
        }
    }

    Ok(())
}

fn unsupported(platform: &str, bundle_type: &str) {
    warn!(
        "Not bundling `{bundle_type}` as creating it is only supported on {platform} (for now?)."
    );
}
