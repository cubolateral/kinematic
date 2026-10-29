//! Shared editor components. Application state and rendering backends stay with the caller.

pub mod canvas;
pub mod code;
pub mod controls;
pub mod diagnostics;
pub mod icons;
pub mod lists;
pub mod numbers;
pub mod properties;
pub mod theme;
pub mod timeline;
pub mod tree;
pub mod widgets;
pub mod windows;

#[cfg(test)]
static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
