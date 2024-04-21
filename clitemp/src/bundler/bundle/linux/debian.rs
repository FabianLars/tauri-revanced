// The structure of a Debian package looks something like this:
//
// foobar_1.2.3_i386.deb   # Actually an ar archive
//     debian-binary           # Specifies deb format version (2.0 in our case)
//     control.tar.gz          # Contains files controlling the installation:
//         control                  # Basic package metadata
//         md5sums                  # Checksums for files in data.tar.gz below
//         postinst                 # Post-installation script (optional)
//         prerm                    # Pre-uninstallation script (optional)
//     data.tar.gz             # Contains files to be installed:
//         usr/bin/foobar                            # Binary executable file
//         usr/share/applications/foobar.desktop     # Desktop file (for apps)
//         usr/share/icons/hicolor/...               # Icon files (for apps)
//         usr/lib/foobar/...                        # Other resource files
//
// For cargo-bundle, we put bundle resource files under /usr/lib/package_name/,
// and then generate the desktop file and control file from the bundle
// metadata, as well as generating the md5sums file.  Currently we do not
// generate postinst or prerm files.

use crate::bundler::bundle::Settings;

use super::{super::common, freedesktop};
use anyhow::Context;

use std::path::{Path, PathBuf};

/// Generate the debian data folders and files.
pub fn generate_data(
    settings: &Settings,
    package_dir: &Path,
) -> crate::bundler::Result<(PathBuf, Vec<freedesktop::Icon>)> {
    // Generate data files.
    let data_dir = package_dir.join("data");
    let bin_dir = data_dir.join("usr/bin");

    for bin in settings.binaries() {
        let bin_path = settings.binary_path(bin);
        common::copy_file(&bin_path, bin_dir.join(bin.name()))
            .with_context(|| format!("Failed to copy binary from {bin_path:?}"))?;
    }

    copy_resource_files(settings, &data_dir).with_context(|| "Failed to copy resource files")?;

    settings
        .copy_binaries(&bin_dir)
        .with_context(|| "Failed to copy external binaries")?;

    let icons = freedesktop::copy_icon_files(settings, &data_dir)
        .with_context(|| "Failed to create icon files")?;
    freedesktop::generate_desktop_file(settings, &None, &data_dir)
        .with_context(|| "Failed to create desktop file")?;

    Ok((data_dir, icons))
}

/// Copy the bundle's resource files into an appropriate directory under the
/// `data_dir`.
fn copy_resource_files(settings: &Settings, data_dir: &Path) -> crate::bundler::Result<()> {
    let resource_dir = data_dir.join("usr/lib").join(settings.main_binary_name());
    settings.copy_resources(&resource_dir)
}
