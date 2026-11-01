//! Short presentation transitions, independent of scene playback.

use dear_imgui_rs::Ui;

pub const CONTROL_DURATION: f32 = 0.110;
pub const OVERLAY_DURATION: f32 = 0.150;

mod overlays;
pub use overlays::Overlays;

/// Animates a scalar using a stable ID in the current window and ID scope.
/// Missing frames restart from `initial`; zero duration applies the target immediately.
pub fn transition(ui: &Ui, id: &str, target: f32, initial: f32, duration: f32) -> f32 {
    let _scope = ui.push_id(id);
    let value_key = ui.get_id("##MotionValue");
    let target_key = ui.get_id("##MotionTarget");
    let start_key = ui.get_id("##MotionStart");
    let elapsed_key = ui.get_id("##MotionElapsed");
    let frame_key = ui.get_id("##MotionFrame");
    let frame = ui.frame_count() as i32;
    ui.with_current_state_storage(|mut storage| {
        let previous_frame = storage.get_int(frame_key, -1);
        let fresh = previous_frame < frame - 1;
        let value = if fresh {
            initial
        } else {
            storage.get_float(value_key, initial)
        };
        let changed = fresh || storage.get_float(target_key, initial) != target;
        let start = if changed {
            value
        } else {
            storage.get_float(start_key, initial)
        };
        let elapsed = if changed {
            0.0
        } else {
            storage.get_float(elapsed_key, 0.0)
        } + if previous_frame == frame {
            0.0
        } else {
            ui.io().delta_time()
        };
        let value = interpolate(start, target, elapsed, duration);
        storage.set_float(value_key, value);
        storage.set_float(target_key, target);
        storage.set_float(start_key, start);
        storage.set_float(elapsed_key, elapsed.min(duration.max(0.0)));
        storage.set_int(frame_key, frame);
        value
    })
}

pub fn mix(from: [f32; 4], to: [f32; 4], amount: f32) -> [f32; 4] {
    std::array::from_fn(|index| from[index] + (to[index] - from[index]) * amount)
}

fn interpolate(from: f32, to: f32, elapsed: f32, duration: f32) -> f32 {
    let progress = if duration <= 0.0 {
        1.0
    } else {
        (elapsed / duration).clamp(0.0, 1.0)
    };
    let eased = 1.0 - (1.0 - progress).powi(3);
    from + (to - from) * eased
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transitions_finish_on_time_regardless_of_frame_rate() {
        for fps in [30, 60, 144] {
            let mut elapsed = 0.0;
            let mut previous = 0.0;
            while elapsed < OVERLAY_DURATION {
                elapsed += 1.0 / fps as f32;
                let value = interpolate(0.0, 1.0, elapsed, OVERLAY_DURATION);
                assert!(value >= previous && value <= 1.0);
                previous = value;
            }
            assert_eq!(previous, 1.0);
            assert!(elapsed < OVERLAY_DURATION + 1.0 / fps as f32);
        }
        assert_eq!(interpolate(0.0, 1.0, 0.0, 0.0), 1.0);
    }
}
