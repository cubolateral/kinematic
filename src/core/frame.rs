pub(crate) fn frame_index(time: f32, fps: u32) -> u64 {
    let frames = time.max(0.0) * fps.max(1) as f32;
    (frames + f32::EPSILON * frames.abs() * 4.0).floor() as u64
}

pub(crate) fn frame_dt(fps: u32) -> f32 {
    1.0 / fps.max(1) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_time_to_a_stable_frame_index() {
        assert_eq!(frame_index(0.0, 60), 0);
        assert_eq!(frame_index(1.0 / 60.0, 60), 1);
        assert_eq!(frame_index(0.5, 60), 30);
        assert_eq!(frame_index(-1.0, 60), 0);
    }
}
