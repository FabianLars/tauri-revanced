use tauri_utils::display_path;

use std::{
    fs::{self, File},
    io::{prelude::*, Write},
    path::{Path, PathBuf},
};

use anyhow::Context;
use zip::write::FileOptions;

use super::{common, Bundle, Settings};

// Build update
pub fn bundle_project(
    settings: &Settings,
    bundles: &[Bundle],
) -> crate::bundler::Result<Vec<PathBuf>> {
    let target_os = settings
        .target()
        .split('-')
        .nth(2)
        .unwrap_or(std::env::consts::OS)
        .replace("darwin", "macos");

    #[cfg(target_os = "macos")]
    return bundle_update_macos(bundles);
    #[cfg(target_os = "linux")]
    return bundle_update_linux(bundles);

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        log::error!("Current platform does not support updates");
        Ok(vec![])
    }
}

// Create simple update-macos.tar.gz
// This is the Mac OS App packaged
#[cfg(target_os = "macos")]
fn bundle_update_macos(bundles: &[Bundle]) -> crate::bundler::Result<Vec<PathBuf>> {
    use std::ffi::OsStr;

    // find our .app or rebuild our bundle
    if let Some(source_path) = bundles
        .iter()
        .filter(|bundle| bundle.package_type == PackageType::MacOsBundle)
        .find_map(|bundle| {
            bundle
                .bundle_paths
                .iter()
                .find(|path| path.extension() == Some(OsStr::new("app")))
        })
    {
        // add .tar.gz to our path
        let osx_archived = format!("{}.tar.gz", source_path.display());
        let osx_archived_path = PathBuf::from(&osx_archived);

        // Create our gzip file (need to send parent)
        // as we walk the source directory (source isnt added)
        create_tar(source_path, &osx_archived_path)
            .with_context(|| "Failed to tar.gz update directory")?;

        log::info!(action = "Bundling"; "{} ({})", osx_archived, display_path(&osx_archived_path));

        Ok(vec![osx_archived_path])
    } else {
        Err(crate::bundler::Error::UnableToFindProject)
    }
}

// Create simple update-linux_<arch>.tar.gz
// Including the AppImage
// Right now in linux we hot replace the bin and request a restart
// No assets are replaced
#[cfg(target_os = "linux")]
fn bundle_update_linux(bundles: &[Bundle]) -> crate::bundler::Result<Vec<PathBuf>> {
    use std::ffi::OsStr;

    // build our app actually we support only appimage on linux
    if let Some(source_path) = bundles
        .iter()
        .filter(|bundle| bundle.package_type == PackageType::AppImage)
        .find_map(|bundle| {
            bundle
                .bundle_paths
                .iter()
                .find(|path| path.extension() == Some(OsStr::new("AppImage")))
        })
    {
        // add .tar.gz to our path
        let appimage_archived = format!("{}.tar.gz", source_path.display());
        let appimage_archived_path = PathBuf::from(&appimage_archived);

        // Create our gzip file
        create_tar(source_path, &appimage_archived_path)
            .with_context(|| "Failed to tar.gz update directory")?;

        log::info!(action = "Bundling"; "{} ({})", appimage_archived, display_path(&appimage_archived_path));

        Ok(vec![appimage_archived_path])
    } else {
        Err(crate::bundler::Error::UnableToFindProject)
    }
}

pub fn create_zip(src_file: &Path, dst_file: &Path) -> crate::bundler::Result<PathBuf> {
    let parent_dir = dst_file.parent().expect("No data in parent");
    fs::create_dir_all(parent_dir)?;
    let writer = common::create_file(dst_file)?;

    let file_name = src_file
        .file_name()
        .expect("Can't extract file name from path");

    let mut zip = zip::ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(0o755);

    zip.start_file(file_name.to_string_lossy(), options)?;
    let mut f = File::open(src_file)?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;
    zip.write_all(&buffer)?;
    buffer.clear();

    Ok(dst_file.to_owned())
}

#[cfg(not(target_os = "windows"))]
fn create_tar(src_dir: &Path, dest_path: &Path) -> crate::bundler::Result<PathBuf> {
    use flate2::{write::GzEncoder, Compression};

    let dest_file = common::create_file(dest_path)?;
    let gzip_encoder = GzEncoder::new(dest_file, Compression::default());

    let gzip_encoder = create_tar_from_src(src_dir, gzip_encoder)?;

    let mut dest_file = gzip_encoder.finish()?;
    dest_file.flush()?;
    Ok(dest_path.to_owned())
}

#[cfg(target_os = "macos")]
fn create_tar_from_src<P: AsRef<Path>, W: Write>(
    src_dir: P,
    dest_file: W,
) -> crate::bundler::Result<W> {
    let src_dir = src_dir.as_ref();
    let mut builder = tar::Builder::new(dest_file);
    builder.follow_symlinks(false);
    builder.append_dir_all(src_dir.file_name().expect("Path has no file_name"), src_dir)?;
    builder.into_inner().map_err(Into::into)
}

#[cfg(target_os = "linux")]
fn create_tar_from_src<P: AsRef<Path>, W: Write>(
    src_dir: P,
    dest_file: W,
) -> crate::bundler::Result<W> {
    let src_dir = src_dir.as_ref();
    let mut tar_builder = tar::Builder::new(dest_file);

    // validate source type
    let file_type = fs::metadata(src_dir).expect("Can't read source directory");
    // if it's a file don't need to walkdir
    if file_type.is_file() {
        let mut src_file = fs::File::open(src_dir)?;
        let file_name = src_dir
            .file_name()
            .expect("Can't extract file name from path");

        tar_builder.append_file(file_name, &mut src_file)?;
    } else {
        for entry in walkdir::WalkDir::new(src_dir) {
            let entry = entry?;
            let src_path = entry.path();
            if src_path == src_dir {
                continue;
            }

            // We add the .parent() because example if we send a path
            // /dev/src-tauri/target/debug/bundle/osx/app.app
            // We need a tar with app.app/<...> (source root folder should be included)
            // safe to unwrap: the path has a parent
            let dest_path = src_path.strip_prefix(src_dir.parent().unwrap())?;
            if entry.file_type().is_dir() {
                tar_builder.append_dir(dest_path, src_path)?;
            } else {
                let mut src_file = fs::File::open(src_path)?;
                tar_builder.append_file(dest_path, &mut src_file)?;
            }
        }
    }
    let dest_file = tar_builder.into_inner()?;
    Ok(dest_file)
}
