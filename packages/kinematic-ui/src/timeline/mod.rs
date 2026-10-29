//! Timeline geometry and painting; callers own scenes, clips, and playback.

pub mod layout;
pub mod metrics;
pub mod painting;
pub mod ruler;
pub mod transport;
pub mod waveform;

pub use layout::{Layout, TimeRange};
pub use ruler::format_time;
