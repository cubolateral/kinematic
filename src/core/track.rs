use crate::core::{
    AnimatorHandle, Easing, SceneWorld, Tween,
    types::{Color, Vector2},
};

/// Getter function used by a track to read its current value from the ECS world.
pub type TrackGetter = fn(&hecs::World, hecs::Entity) -> TrackValue;
/// Setter function used by a track to write the interpolated value back to the ECS world.
pub type TrackSetter = fn(&hecs::World, hecs::Entity, TrackValue);
/// Numeric identifier for a component field track.
pub type TrackId = u32;

#[derive(Debug)]
pub(crate) struct Keyframe {
    pub time: f32,
    pub value: TrackValue,
    pub easing: Option<Easing>,
}

#[derive(Debug)]
pub(crate) struct Track {
    pub info: &'static TrackInfo,
    pub keyframes: Vec<Keyframe>,
    current_tween_range: (f32, f32),
    current_tween_start: usize,
}

impl Track {
    /// Creates an empty track for a single component field.
    pub fn new(info: &'static TrackInfo) -> Self {
        Self {
            info,
            keyframes: vec![],
            current_tween_range: (0.0, 0.0),
            current_tween_start: 0,
        }
    }

    pub fn update(&mut self, world: &hecs::World, entity: hecs::Entity, time: f32) {
        let set = self.info.set;

        match self.find_keyframes(time) {
            (Some(left), Some(right)) => {
                // Prevents division by zero.
                if left.time == right.time {
                    set(world, entity, left.value.clone());
                    return;
                }

                let t = match left.easing {
                    Some(easing) => easing.evaluate((time - left.time) / (right.time - left.time)),
                    None => {
                        set(world, entity, left.value.clone());
                        return;
                    }
                };

                set(world, entity, left.value.lerp(&right.value, t));
            }
            (Some(left), None) => set(world, entity, left.value.clone()),
            (None, Some(right)) => set(world, entity, right.value.clone()),
            (None, None) => {}
        }
    }

    pub fn set_keyframe(&mut self, time: f32, value: TrackValue, easing: Option<Easing>) {
        self.clear_current_tween_range();

        let length = self.keyframes.len();

        if let Some(last) = self.keyframes.last_mut() {
            // Keyframes are appended in time order because runtime lookup assumes
            // the list is monotonic and can be searched with a simple binary walk.
            assert!(
                time >= last.time,
                "Keyframes must be appended in non-decreasing time order."
            );

            if time == last.time {
                if length == 1 {
                    // Special case: this happens when the first tween is created with
                    // `duration == 0`. In that case, the value must stay at its original
                    // state before the target time, so we keep the first keyframe as the
                    // initial value and let the new one represent the instant change.
                    last.time = 0.0;
                    last.easing = None;
                } else {
                    // Same timestamp as the latest keyframe means this is an update,
                    // not a new segment. Replace the value/easing in place.
                    last.value = value;
                    last.easing = easing;
                    return;
                }
            }
        }

        // Append a new keyframe. This is the common path for the first write and
        // for any later keyframe that advances time.
        self.keyframes.push(Keyframe {
            time,
            value,
            easing,
        });
    }

    fn find_keyframes(&mut self, time: f32) -> (Option<&Keyframe>, Option<&Keyframe>) {
        if self.keyframes.is_empty() {
            return (None, None);
        }

        let (start, end) = self.current_tween_range;

        if start < end && start <= time && time <= end {
            if start == 0.0 && end == self.keyframes[0].time {
                return (None, self.keyframes.first());
            }

            if start == self.keyframes.last().unwrap().time && end == f32::INFINITY {
                return (self.keyframes.last(), None);
            }

            let left = &self.keyframes[self.current_tween_start];
            let right = &self.keyframes[self.current_tween_start + 1];

            return (Some(left), Some(right));
        }

        self.clear_current_tween_range();

        let mut left = 0usize;
        let mut right = self.keyframes.len();

        while left < right {
            let mid = left + (right - left) / 2;
            let t = self.keyframes[mid].time;

            if t == time {
                return (Some(&self.keyframes[mid]), Some(&self.keyframes[mid]));
            } else if t < time {
                left = mid + 1;
            } else {
                right = mid;
            }
        }

        let left_index = left.checked_sub(1);
        let right_index = (left < self.keyframes.len()).then_some(left);

        match (left_index, right_index) {
            (None, Some(right)) if time >= 0.0 && self.keyframes[right].time > 0.0 => {
                self.current_tween_range = (0.0, self.keyframes[right].time);
            }
            (Some(left), None) if self.keyframes[left].time < f32::INFINITY => {
                self.current_tween_range = (self.keyframes[left].time, f32::INFINITY);
            }
            (Some(left), Some(right))
                if self.keyframes[left].easing.is_some()
                    && self.keyframes[left].time < self.keyframes[right].time =>
            {
                self.current_tween_range = (self.keyframes[left].time, self.keyframes[right].time);
                self.current_tween_start = left;
            }
            _ => {}
        }

        (
            left_index.map(|index| &self.keyframes[index]),
            right_index.map(|index| &self.keyframes[index]),
        )
    }

    fn clear_current_tween_range(&mut self) {
        self.current_tween_range = (0.0, 0.0);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TrackValue {
    F32(f32),
    Vector2(Vector2),
    Color(Color),
    String(String),
}

impl TrackValue {
    pub fn lerp(&self, to: &Self, t: f32) -> Self {
        match (self, to) {
            (Self::F32(a), Self::F32(b)) => Self::F32(a + (b - a) * t),
            (Self::Vector2(a), Self::Vector2(b)) => Self::Vector2(a + (b - a) * t),
            (Self::Color(a), Self::Color(b)) => {
                let [ar, ag, ab, aa] = a.rgba();
                let [br, bg, bb, ba] = b.rgba();

                Self::Color(Color::new(
                    ar + (br - ar) * t,
                    ag + (bg - ag) * t,
                    ab + (bb - ab) * t,
                    aa + (ba - aa) * t,
                ))
            }
            (Self::String(a), Self::String(b)) => {
                let t = t.clamp(0.0, 1.0);

                if t <= 0.0 {
                    return Self::String(a.clone());
                }

                if t >= 1.0 {
                    return Self::String(b.clone());
                }

                let from: Vec<char> = a.chars().collect();
                let to: Vec<char> = b.chars().collect();

                let len = from.len().max(to.len());

                if len == 0 {
                    return Self::String(String::new());
                }

                let mut result = String::new();

                for i in 0..len {
                    let local_t = (t * len as f32 - i as f32).clamp(0.0, 1.0);

                    let character = if local_t < 0.5 {
                        from.get(i)
                    } else {
                        to.get(i)
                    };

                    if let Some(character) = character {
                        result.push(*character);
                    }
                }

                Self::String(result)
            }
            _ => panic!("Track values must have the same type."),
        }
    }
}

impl std::fmt::Display for TrackValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::F32(value) => write!(f, "{value:.2}"),
            Self::Vector2(value) => write!(f, "[{:.2}, {:.2}]", value.x, value.y),
            Self::Color(value) => {
                let [r, g, b, a] = value.rgba();
                write!(f, "[{r:.2}, {g:.2}, {b:.2}, {a:.2}]")
            }
            Self::String(value) => f.write_str(value),
        }
    }
}

/// Converts a component field value to and from its runtime track representation.
///
/// Every field marked with `#[track]` must implement this trait.
pub trait TrackValueType: Clone {
    /// Input accepted by the generated track tween method.
    type Input: Into<Self>;

    /// Converts this typed value into the value stored by an animation track.
    fn into_track_value(self) -> TrackValue;

    /// Converts an animation track value back into this typed value.
    fn from_track_value(value: TrackValue) -> Option<Self>;
}

impl TrackValueType for f32 {
    type Input = f32;

    fn into_track_value(self) -> TrackValue {
        TrackValue::F32(self)
    }

    fn from_track_value(value: TrackValue) -> Option<Self> {
        match value {
            TrackValue::F32(value) => Some(value),
            _ => None,
        }
    }
}

impl TrackValueType for Vector2 {
    type Input = Vector2;

    fn into_track_value(self) -> TrackValue {
        TrackValue::Vector2(self)
    }

    fn from_track_value(value: TrackValue) -> Option<Self> {
        match value {
            TrackValue::Vector2(value) => Some(value),
            _ => None,
        }
    }
}

impl TrackValueType for Color {
    type Input = Color;

    fn into_track_value(self) -> TrackValue {
        TrackValue::Color(self)
    }

    fn from_track_value(value: TrackValue) -> Option<Self> {
        match value {
            TrackValue::Color(value) => Some(value),
            _ => None,
        }
    }
}

impl TrackValueType for String {
    type Input = String;

    fn into_track_value(self) -> TrackValue {
        TrackValue::String(self)
    }

    fn from_track_value(value: TrackValue) -> Option<Self> {
        match value {
            TrackValue::String(value) => Some(value),
            _ => None,
        }
    }
}

/// Typed interface for updating a single tracked component field.
pub struct TrackHandle<T: TrackValueType> {
    world: SceneWorld,
    entity: hecs::Entity,
    type_id: std::any::TypeId,
    info: &'static TrackInfo,
    get: fn(&hecs::World, hecs::Entity) -> T,
    replace: fn(&mut hecs::World, hecs::Entity, T) -> T,
    animator: AnimatorHandle,
}

impl<T: TrackValueType> TrackHandle<T> {
    /// Creates a typed handle using generated component accessors.
    pub fn new(
        world: SceneWorld,
        entity: hecs::Entity,
        type_id: std::any::TypeId,
        info: &'static TrackInfo,
        get: fn(&hecs::World, hecs::Entity) -> T,
        replace: fn(&mut hecs::World, hecs::Entity, T) -> T,
        animator: AnimatorHandle,
    ) -> Self {
        Self {
            world,
            entity,
            type_id,
            info,
            get,
            replace,
            animator,
        }
    }

    /// Returns the current component field value without creating a tween.
    pub fn get(&self) -> T {
        let world = self.world.borrow();
        (self.get)(&world, self.entity)
    }

    /// Sets the field target and returns the corresponding tween.
    pub fn set(&self, value: T) -> Tween {
        let old_value = {
            let mut world = self.world.borrow_mut();
            (self.replace)(&mut world, self.entity, value.clone())
        };

        self.tween(old_value, value)
    }

    fn update(&self, update: impl FnOnce(T) -> T) -> Tween {
        let (old_value, new_value) = {
            let world = self.world.borrow();
            let old_value = (self.get)(&world, self.entity);
            let new_value = update(old_value.clone());
            drop(world);
            let mut world = self.world.borrow_mut();
            (self.replace)(&mut world, self.entity, new_value.clone());
            (old_value, new_value)
        };

        self.tween(old_value, new_value)
    }

    fn tween(&self, from: T, to: T) -> Tween {
        Tween::new(
            self.entity,
            self.type_id,
            self.info,
            from.into_track_value(),
            to.into_track_value(),
            self.animator.clone(),
        )
    }
}

impl TrackHandle<Vector2> {
    /// Sets the horizontal coordinate while preserving the vertical coordinate.
    pub fn x(&self, value: f32) -> Tween {
        self.update(|mut position| {
            position.x = value;
            position
        })
    }

    /// Sets the vertical coordinate while preserving the horizontal coordinate.
    pub fn y(&self, value: f32) -> Tween {
        self.update(|mut position| {
            position.y = value;
            position
        })
    }
}

impl TrackHandle<Color> {
    /// Sets the red channel while preserving the remaining color channels.
    pub fn r(&self, value: f32) -> Tween {
        self.update(|mut color| {
            color.r = value;
            color
        })
    }

    /// Sets the green channel while preserving the remaining color channels.
    pub fn g(&self, value: f32) -> Tween {
        self.update(|mut color| {
            color.g = value;
            color
        })
    }

    /// Sets the blue channel while preserving the remaining color channels.
    pub fn b(&self, value: f32) -> Tween {
        self.update(|mut color| {
            color.b = value;
            color
        })
    }

    /// Sets the alpha channel while preserving the remaining color channels.
    pub fn a(&self, value: f32) -> Tween {
        self.update(|mut color| {
            color.a = value;
            color
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_TRACK_INFO: TrackInfo = TrackInfo {
        id: 0,
        name: "value",
        get: |_, _| TrackValue::F32(0.0),
        set: |_, _, _| {},
    };

    #[test]
    fn caches_the_current_tween_range() {
        let mut track = tween_track();

        let _ = track.find_keyframes(5.0);
        assert_eq!(track.current_tween_range, (0.0, 10.0));

        let _ = track.find_keyframes(8.0);
        assert_eq!(track.current_tween_range, (0.0, 10.0));

        let _ = track.find_keyframes(15.0);
        assert_eq!(track.current_tween_range, (10.0, 20.0));

        let _ = track.find_keyframes(5.0);
        assert_eq!(track.current_tween_range, (0.0, 10.0));
    }

    #[test]
    fn caches_the_range_before_the_first_keyframe() {
        let mut track = offset_tween_track();

        let (left, right) = track.find_keyframes(1.0);
        assert!(left.is_none());
        assert_eq!(right.unwrap().time, 3.0);
        assert_eq!(track.current_tween_range, (0.0, 3.0));

        let (left, right) = track.find_keyframes(2.0);
        assert!(left.is_none());
        assert_eq!(right.unwrap().time, 3.0);
        assert_eq!(track.current_tween_range, (0.0, 3.0));
    }

    #[test]
    fn caches_the_range_after_the_last_keyframe() {
        let mut track = tween_track();

        let (left, right) = track.find_keyframes(25.0);
        assert_eq!(left.unwrap().time, 20.0);
        assert!(right.is_none());
        assert_eq!(track.current_tween_range, (20.0, f32::INFINITY));

        let (left, right) = track.find_keyframes(30.0);
        assert_eq!(left.unwrap().time, 20.0);
        assert!(right.is_none());
        assert_eq!(track.current_tween_range, (20.0, f32::INFINITY));
    }

    #[test]
    fn invalidates_the_cached_tween_range_when_adding_a_keyframe() {
        let mut track = tween_track();

        let _ = track.find_keyframes(5.0);
        assert_eq!(track.current_tween_range, (0.0, 10.0));

        track.set_keyframe(30.0, TrackValue::F32(30.0), None);

        assert_eq!(track.current_tween_range, (0.0, 0.0));
    }

    #[test]
    fn interpolates_vector_values() {
        let value = TrackValue::Vector2(Vector2::ZERO)
            .lerp(&TrackValue::Vector2(Vector2::new(10.0, 20.0)), 0.5);

        assert!(matches!(value, TrackValue::Vector2(vector) if vector == Vector2::new(5.0, 10.0)));
    }

    #[test]
    fn interpolates_color_values() {
        let value = TrackValue::Color(Color::new(0.0, 0.0, 0.0, 0.0))
            .lerp(&TrackValue::Color(Color::new(1.0, 0.5, 0.25, 1.0)), 0.5);

        assert_eq!(value, TrackValue::Color(Color::new(0.5, 0.25, 0.125, 0.5)));
    }

    #[test]
    fn interpolates_string_values_by_revealing_unicode_characters() {
        let from = TrackValue::String("Source!".to_owned());
        let to = TrackValue::String("Aé🦀!".to_owned());

        assert_eq!(from.lerp(&to, 0.0), from);
        assert_eq!(
            from.lerp(&to, 0.5),
            TrackValue::String("Aé🦀!ce!".to_owned())
        );
        assert_eq!(from.lerp(&to, 1.0), to);
    }

    #[test]
    fn converts_string_track_values() {
        let value = "Kinematic!".to_owned();
        let track_value = value.clone().into_track_value();

        assert_eq!(String::from_track_value(track_value), Some(value));
    }

    fn tween_track() -> Track {
        let mut track = Track::new(&TEST_TRACK_INFO);
        track.set_keyframe(0.0, TrackValue::F32(0.0), Some(Easing::Linear));
        track.set_keyframe(10.0, TrackValue::F32(10.0), Some(Easing::Linear));
        track.set_keyframe(20.0, TrackValue::F32(20.0), None);
        track
    }

    fn offset_tween_track() -> Track {
        let mut track = Track::new(&TEST_TRACK_INFO);
        track.set_keyframe(3.0, TrackValue::F32(3.0), Some(Easing::Linear));
        track.set_keyframe(10.0, TrackValue::F32(10.0), None);
        track
    }
}

#[derive(Debug)]
pub struct TrackInfo {
    /// Stable track id inside the component type.
    pub id: TrackId,
    /// Human-readable field name used by tooling and debugging.
    pub name: &'static str,
    /// Reads the current value of the tracked field.
    pub get: TrackGetter,
    /// Writes an interpolated value back to the field.
    pub set: TrackSetter,
}

/// Metadata for a component that exposes one or more tracked fields.
#[derive(Clone, Copy)]
pub struct TrackableInfo {
    /// Name of the trackable component, usually the Rust type name.
    pub name: &'static str,
    /// Returns the static list of tracked fields for the component.
    pub get: fn() -> &'static [TrackInfo],
}

/// Trait implemented by types that expose animatable fields.
pub trait Trackable {
    /// Internal field layer added to generated object handlers.
    type HandlerFields<Next>;

    /// Builds this component's tracked fields around the next handler layer.
    fn handler_fields<Next>(
        world: SceneWorld,
        entity: hecs::Entity,
        animator: AnimatorHandle,
        next: Next,
    ) -> Self::HandlerFields<Next>;

    /// Returns metadata for a tracked field id.
    fn track(id: TrackId) -> &'static TrackInfo;

    /// Returns metadata for the whole trackable component.
    fn info() -> &'static TrackableInfo;
}
