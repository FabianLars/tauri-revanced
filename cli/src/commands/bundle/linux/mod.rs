// This is the entry point for Linux specific bundles
// Only AppImages and flatpak (via flathub) will be supported.
// For the latter we'll only generate a template (for now?).

pub mod appimage;
pub mod flatpak;
