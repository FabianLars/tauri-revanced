// This is the entry point for Windows specific bundles.
// Not sure which bundle formats i'll add, initially i only wanted msix but idk, maybe i'll look into rust-msi too.
// wix5 is somewhat planned for upstream tauri already but maybe i'll try it here first.
// Exploring how far we can get with 1) just an exe and 2) just a zip may be cool too ig.

pub mod msi;
pub mod msix;
pub mod wix;
pub mod zip;
