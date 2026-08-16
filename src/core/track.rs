use crate::core::{Easing, Vector2};

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
    pub track_setter: TrackSetter,
    pub keyframes: Vec<Keyframe>,
}

impl Track {
    /// Creates an empty track for a single component field.
    pub fn new(track_setter: TrackSetter) -> Self {
        Self {
            track_setter,
            keyframes: vec![],
        }
    }

    pub fn update(&self, world: &hecs::World, entity: hecs::Entity, time: f32) {
        match self.find_keyframes(time) {
            (Some(left), Some(right)) => {
                // Prevents division by zero.
                if left.time == right.time {
                    (self.track_setter)(world, entity, left.value);
                    return;
                }

                let t = match left.easing {
                    Some(easing) => easing.evaluate((time - left.time) / (right.time - left.time)),
                    None => {
                        (self.track_setter)(world, entity, left.value);
                        return;
                    }
                };

                (self.track_setter)(world, entity, left.value.lerp(right.value, t));
            }
            (Some(left), None) => (self.track_setter)(world, entity, left.value),
            (None, Some(right)) => (self.track_setter)(world, entity, right.value),
            (None, None) => {}
        }
    }

    pub fn set_keyframe(&mut self, time: f32, value: TrackValue, easing: Option<Easing>) {
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

    fn find_keyframes(&self, time: f32) -> (Option<&Keyframe>, Option<&Keyframe>) {
        if self.keyframes.is_empty() {
            return (None, None);
        }

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

        (
            left.checked_sub(1).map(|i| &self.keyframes[i]),
            self.keyframes.get(left),
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TrackValue {
    F32(f32),
    Vector2(Vector2),
}

impl TrackValue {
    pub fn lerp(self, to: Self, t: f32) -> Self {
        match (self, to) {
            (Self::F32(a), Self::F32(b)) => Self::F32(a + (b - a) * t),
            (Self::Vector2(a), Self::Vector2(b)) => Self::Vector2(a + (b - a) * t),
            _ => panic!("Track values must have the same type."),
        }
    }
}

impl std::fmt::Display for TrackValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::F32(value) => write!(f, "{value:.2}"),
            Self::Vector2(value) => write!(f, "[{:.2}, {:.2}]", value.x, value.y),
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
    type Input = [f32; 2];

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

/// Typed interface for updating a single tracked component field.
pub struct TrackHandle<'a, T: TrackValueType> {
    scene: std::rc::Rc<std::cell::RefCell<&'a mut crate::core::Scene>>,
    entity: hecs::Entity,
    type_id: std::any::TypeId,
    track_id: TrackId,
    track_setter: TrackSetter,
    get: fn(&crate::core::Scene, hecs::Entity) -> T,
    replace: fn(&mut crate::core::Scene, hecs::Entity, T) -> T,
}

impl<'a, T: TrackValueType> TrackHandle<'a, T> {
    /// Creates a typed handle using generated component accessors.
    pub fn new(
        scene: std::rc::Rc<std::cell::RefCell<&'a mut crate::core::Scene>>,
        entity: hecs::Entity,
        type_id: std::any::TypeId,
        track_id: TrackId,
        track_setter: TrackSetter,
        get: fn(&crate::core::Scene, hecs::Entity) -> T,
        replace: fn(&mut crate::core::Scene, hecs::Entity, T) -> T,
    ) -> Self {
        Self {
            scene,
            entity,
            type_id,
            track_id,
            track_setter,
            get,
            replace,
        }
    }

    /// Sets the field target and returns the corresponding tween.
    pub fn set(self, value: T) -> crate::core::Tween {
        let old_value = {
            let mut scene = self.scene.borrow_mut();
            (self.replace)(&mut scene, self.entity, value.clone())
        };

        self.tween(old_value, value)
    }

    fn update(self, update: impl FnOnce(T) -> T) -> crate::core::Tween {
        let (old_value, new_value) = {
            let mut scene = self.scene.borrow_mut();
            let old_value = (self.get)(&scene, self.entity);
            let new_value = update(old_value.clone());
            (self.replace)(&mut scene, self.entity, new_value.clone());
            (old_value, new_value)
        };

        self.tween(old_value, new_value)
    }

    fn tween(self, from: T, to: T) -> crate::core::Tween {
        crate::core::Tween::new(
            self.entity,
            self.type_id,
            self.track_id,
            self.track_setter,
            from.into_track_value(),
            to.into_track_value(),
        )
    }
}

impl<'a> TrackHandle<'a, Vector2> {
    /// Sets the horizontal coordinate while preserving the vertical coordinate.
    pub fn x(self, value: f32) -> crate::core::Tween {
        self.update(|mut position| {
            position.x = value;
            position
        })
    }

    /// Sets the vertical coordinate while preserving the horizontal coordinate.
    pub fn y(self, value: f32) -> crate::core::Tween {
        self.update(|mut position| {
            position.y = value;
            position
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolates_vector_values() {
        let value = TrackValue::Vector2(Vector2::ZERO)
            .lerp(TrackValue::Vector2(Vector2::new(10.0, 20.0)), 0.5);

        assert!(matches!(value, TrackValue::Vector2(vector) if vector == Vector2::new(5.0, 10.0)));
    }
}

pub struct TrackInfo {
    /// Stable track id inside the component type.
    pub id: TrackId,
    /// Human-readable field name used by tooling and debugging.
    pub name: &'static str,
    /// Reads the current value of the tracked field.
    pub get: fn(&hecs::World, hecs::Entity) -> TrackValue,
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
    /// Per-type handle returned by the generated `handle(...)` helper.
    type Handle<'a>
    where
        Self: 'a;

    /// Builds a handle around an entity stored in the scene world.
    fn handle<'a>(scene: &'a mut crate::core::Scene, entity: hecs::Entity) -> Self::Handle<'a>;

    /// Returns metadata for a tracked field id.
    fn track(id: TrackId) -> &'static TrackInfo;

    /// Returns metadata for the whole trackable component.
    fn info() -> &'static TrackableInfo;
}
