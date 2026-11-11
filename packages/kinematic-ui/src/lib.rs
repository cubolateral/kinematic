//! Shared editor components. Application state and rendering backends stay with the caller.

pub mod canvas;
pub mod controls;
pub mod diagnostics;
pub mod icons;
pub mod lists;
pub mod menu;
pub mod message_box;
pub mod motion;
pub mod numbers;
pub mod properties;
pub mod shortcut;
pub mod status_bar;
pub mod theme;
pub mod timeline;
pub mod title_bar;
pub mod tree;
pub mod widgets;
pub mod windows;

#[cfg(test)]
static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
