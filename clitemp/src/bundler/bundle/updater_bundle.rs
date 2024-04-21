use std::path::PathBuf;

use super::{Bundle, Settings};

// Build update
pub fn bundle_project(
    _settings: &Settings,
    _bundles: &[Bundle],
) -> crate::bundler::Result<Vec<PathBuf>> {
    /* let target_os = settings
           .target()
           .split('-')
           .nth(2)
           .unwrap_or(std::env::consts::OS)
           .replace("darwin", "macos");
    */
    #[cfg(target_os = "macos")]
    return bundle_update_macos(_bundles);

    #[cfg(not(any(target_os = "macos")))]
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

#[cfg(target_os = "macos")]
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
