#![cfg(desktop)]

//! A module containing menu builder types

mod menu;
pub use menu::MenuBuilder;
mod normal;
pub use normal::MenuItemBuilder;
mod submenu;
pub use submenu::SubmenuBuilder;
mod check;
pub use check::CheckMenuItemBuilder;
mod icon;
pub use icon::IconMenuItemBuilder;
