use crate::core::{
    AnimatorHandle, Easing, SceneWorld, Tween,
    objects::HandlerContext,
    types::{Color, Quad, Quaternion, Vector2, Vector3},
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
    interpolation: TrackInterpolation,
}

#[derive(Debug, Clone, Copy)]
enum TrackInterpolation {
    Value,
    QuaternionAxisAngle { axis: Vector3, angle: f32 },
}

/// A finite cycle evaluated periodically from its scene start.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TrackRepeat {
    pub start: f32,
    pub duration: f32,
}

impl TrackRepeat {
    pub(crate) fn local_time(self, time: f32) -> f32 {
        if time < self.start {
            time
        } else {
            self.start + (time - self.start).rem_euclid(self.duration)
        }
    }
}

#[derive(Debug)]
pub(crate) struct Track {
    pub info: &'static TrackInfo,
    pub keyframes: Vec<Keyframe>,
    pub repeat: Option<TrackRepeat>,
    current_tween_range: (f32, f32),
    current_tween_start: usize,
}

impl Track {
    /// Creates an empty track for a single component field.
    pub fn new(info: &'static TrackInfo) -> Self {
        Self {
            info,
            keyframes: vec![],
            repeat: None,
            current_tween_range: (0.0, 0.0),
            current_tween_start: 0,
        }
    }

    pub fn update(&mut self, world: &hecs::World, entity: hecs::Entity, time: f32) {
        let set = self.info.set;
        let time = self.repeat.map_or(time, |repeat| repeat.local_time(time));
        let (left, right) = self.find_keyframes(time);
        if let Some(value) =
            Self::sample_keyframes(left, right, time).map(|value| self.info.clamp(value))
        {
            set(world, entity, value);
        }
    }

    /// Evaluates this track at `time` without writing the result to the ECS world.
    pub fn sample(&self, time: f32) -> Option<TrackValue> {
        let time = self.repeat.map_or(time, |repeat| repeat.local_time(time));
        let right = self
            .keyframes
            .partition_point(|keyframe| keyframe.time <= time);
        let left = right.checked_sub(1).map(|index| &self.keyframes[index]);
        let right = self.keyframes.get(right);
        Self::sample_keyframes(left, right, time).map(|value| self.info.clamp(value))
    }

    fn sample_keyframes(
        left: Option<&Keyframe>,
        right: Option<&Keyframe>,
        time: f32,
    ) -> Option<TrackValue> {
        match (left, right) {
            (Some(left), Some(right)) => {
                // Prevents division by zero.
                if left.time == right.time {
                    return Some(left.value.clone());
                }

                // Discrete tracks hold their starting value until the segment ends.
                if matches!(
                    (&left.value, &right.value),
                    (TrackValue::Bool(_), TrackValue::Bool(_))
                        | (TrackValue::String(_), TrackValue::String(_))
                ) {
                    let value = if time < right.time {
                        left.value.clone()
                    } else {
                        right.value.clone()
                    };

                    return Some(value);
                }

                let t = match left.easing {
                    Some(easing) => easing.evaluate((time - left.time) / (right.time - left.time)),
                    None => return Some(left.value.clone()),
                };

                let value = match left.interpolation {
                    TrackInterpolation::Value => left.value.lerp(&right.value, t),
                    TrackInterpolation::QuaternionAxisAngle { axis, angle } => {
                        let TrackValue::Quaternion(start) = &left.value else {
                            panic!("Axis-angle interpolation requires a quaternion track.");
                        };
                        TrackValue::Quaternion(normalized_quaternion(
                            normalized_quaternion(*start)
                                * Quaternion::from_axis_angle(axis, angle * t),
                        ))
                    }
                };

                Some(value)
            }
            (Some(left), None) => Some(left.value.clone()),
            (None, Some(right)) => Some(right.value.clone()),
            (None, None) => None,
        }
    }

    /// Appends a tween while preserving discontinuities created by instant changes.
    pub fn add_tween(
        &mut self,
        start_time: f32,
        from: TrackValue,
        to: TrackValue,
        duration: f32,
        easing: Easing,
    ) {
        if duration == 0.0 {
            // Preserve the value before a standalone instant change. If another
            // tween ends now, its final keyframe already provides that value.
            if self
                .keyframes
                .last()
                .is_none_or(|keyframe| keyframe.time < start_time)
            {
                self.set_keyframe(start_time, from, None);
            }

            self.set_keyframe(start_time, to, None);
            return;
        }

        // The starting keyframe owns the easing for the following segment.
        self.set_keyframe(start_time, from, Some(easing));
        self.set_keyframe(start_time + duration, to, None);
    }

    /// Appends a quaternion tween that preserves its axis, direction and winding.
    pub fn add_rotation_tween(
        &mut self,
        start_time: f32,
        from: Quaternion,
        axis: Vector3,
        angle: f32,
        duration: f32,
        easing: Easing,
    ) {
        let from = normalized_quaternion(from);
        let to = normalized_quaternion(from * Quaternion::from_axis_angle(axis, angle));

        if duration == 0.0 {
            self.add_tween(
                start_time,
                TrackValue::Quaternion(from),
                TrackValue::Quaternion(to),
                duration,
                easing,
            );
            return;
        }

        self.set_keyframe_with_interpolation(
            start_time,
            TrackValue::Quaternion(from),
            Some(easing),
            TrackInterpolation::QuaternionAxisAngle { axis, angle },
        );
        self.set_keyframe(start_time + duration, TrackValue::Quaternion(to), None);
    }

    /// Returns all keyframes at `time` in insertion order.
    pub fn keyframes_at(&self, time: f32) -> &[Keyframe] {
        let start = self
            .keyframes
            .partition_point(|keyframe| keyframe.time < time);
        let end = self
            .keyframes
            .partition_point(|keyframe| keyframe.time <= time);

        &self.keyframes[start..end]
    }

    fn set_keyframe(&mut self, time: f32, value: TrackValue, easing: Option<Easing>) {
        self.set_keyframe_with_interpolation(time, value, easing, TrackInterpolation::Value);
    }

    fn set_keyframe_with_interpolation(
        &mut self,
        time: f32,
        value: TrackValue,
        easing: Option<Easing>,
        interpolation: TrackInterpolation,
    ) {
        self.clear_current_tween_range();
        let value = self.info.clamp(value);

        if let Some(last) = self.keyframes.last_mut() {
            // Keyframes are appended in time order because runtime lookup assumes
            // the list is monotonic and can be searched with a simple binary walk.
            assert!(
                time >= last.time,
                "Keyframes must be appended in non-decreasing time order."
            );

            if time == last.time && easing.is_some() && last.value == value {
                // Share continuous endpoints; preserve both values for a jump.
                last.easing = easing;
                last.interpolation = interpolation;
                return;
            }
        }

        // Append a new keyframe. This is the common path for the first write and
        // for any later keyframe that advances time.
        self.keyframes.push(Keyframe {
            time,
            value,
            easing,
            interpolation,
        });
    }

    fn find_keyframes(&mut self, time: f32) -> (Option<&Keyframe>, Option<&Keyframe>) {
        if self.keyframes.is_empty() {
            return (None, None);
        }

        let (start, end) = self.current_tween_range;

        if start < end && start <= time && time < end {
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

        let left = self
            .keyframes
            .partition_point(|keyframe| keyframe.time <= time);

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
    Bool(bool),
    U32(u32),
    I32(i32),
    F32(f32),
    Quad(Quad),
    Vector2(Vector2),
    Vector3(Vector3),
    Quaternion(Quaternion),
    Color(Color),
    String(String),
}

impl TrackValue {
    pub fn lerp(&self, to: &Self, t: f32) -> Self {
        match (self, to) {
            (Self::Bool(a), Self::Bool(b)) => {
                if t < 1.0 {
                    Self::Bool(*a)
                } else {
                    Self::Bool(*b)
                }
            }
            (Self::U32(a), Self::U32(b)) => Self::U32(
                (f64::from(*a) + (f64::from(*b) - f64::from(*a)) * f64::from(t)).round() as u32,
            ),
            (Self::I32(a), Self::I32(b)) => Self::I32(
                (f64::from(*a) + (f64::from(*b) - f64::from(*a)) * f64::from(t)).round() as i32,
            ),
            (Self::F32(a), Self::F32(b)) => Self::F32(a + (b - a) * t),
            (Self::Quad(a), Self::Quad(b)) => Self::Quad(a.lerp(*b, t)),
            (Self::Vector2(a), Self::Vector2(b)) => Self::Vector2(a + (b - a) * t),
            (Self::Vector3(a), Self::Vector3(b)) => Self::Vector3(a.lerp(*b, t)),
            (Self::Quaternion(a), Self::Quaternion(b)) => {
                let a = normalized_quaternion(*a);
                let b = normalized_quaternion(*b);
                Self::Quaternion(a.slerp(b, t).normalize())
            }
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
                if t < 1.0 {
                    Self::String(a.clone())
                } else {
                    Self::String(b.clone())
                }
            }
            _ => panic!("Track values must have the same type."),
        }
    }
}

impl std::fmt::Display for TrackValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(value) => write!(f, "{value}"),
            Self::U32(value) => write!(f, "{value}"),
            Self::I32(value) => write!(f, "{value}"),
            Self::F32(value) => write!(f, "{value:.2}"),
            Self::Quad(value) => write!(
                f,
                "[{:.2}, {:.2}, {:.2}, {:.2}]",
                value.a, value.b, value.c, value.d
            ),
            Self::Vector2(value) => write!(f, "[{:.2}, {:.2}]", value.x, value.y),
            Self::Vector3(v) => write!(f, "[{:.2}, {:.2}, {:.2}]", v.x, v.y, v.z),
            Self::Quaternion(v) => write!(f, "[{:.2}, {:.2}, {:.2}, {:.2}]", v.x, v.y, v.z, v.w),
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

macro_rules! impl_track_value_type {
    ($type:ty, $variant:ident) => {
        impl TrackValueType for $type {
            type Input = $type;

            fn into_track_value(self) -> TrackValue {
                TrackValue::$variant(self)
            }

            fn from_track_value(value: TrackValue) -> Option<Self> {
                match value {
                    TrackValue::$variant(value) => Some(value),
                    _ => None,
                }
            }
        }
    };
}

impl_track_value_type!(bool, Bool);
impl_track_value_type!(u32, U32);
impl_track_value_type!(i32, I32);
impl_track_value_type!(f32, F32);
impl_track_value_type!(Quad, Quad);
impl_track_value_type!(Vector2, Vector2);
impl_track_value_type!(Color, Color);
impl_track_value_type!(Vector3, Vector3);
impl_track_value_type!(Quaternion, Quaternion);
impl_track_value_type!(String, String);

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
        self.clamp((self.get)(&world, self.entity))
    }

    /// Writes the current component field value without creating a tween.
    #[doc(hidden)]
    pub fn set_direct(&self, value: T) {
        let value = self.clamp(value);
        let old_value = {
            let mut world = self.world.borrow_mut();
            (self.replace)(&mut world, self.entity, value)
        };
        self.animator.record_signal_override(
            self.entity,
            self.type_id,
            self.info,
            old_value.into_track_value(),
        );
    }

    /// Sets the field target and returns the corresponding tween.
    pub fn set(&self, value: T) -> Tween {
        self.set_for(value)
    }

    /// Sets the field target for a generated object-specific tween.
    #[doc(hidden)]
    pub fn set_for<Object>(&self, value: T) -> Tween<Object> {
        self.animator.assert_timeline_mutation();
        let value = self.clamp(value);
        let old_value = {
            let mut world = self.world.borrow_mut();
            (self.replace)(&mut world, self.entity, value.clone())
        };

        self.tween(old_value, value)
    }

    #[doc(hidden)]
    pub fn update(&self, update: impl FnOnce(T) -> T) -> Tween {
        self.update_for(update)
    }

    /// Updates the field target for a generated object-specific tween.
    #[doc(hidden)]
    pub fn update_for<Object>(&self, update: impl FnOnce(T) -> T) -> Tween<Object> {
        self.animator.assert_timeline_mutation();
        let (old_value, new_value) = {
            let world = self.world.borrow();
            let old_value = (self.get)(&world, self.entity);
            let new_value = self.clamp(update(old_value.clone()));
            drop(world);
            let mut world = self.world.borrow_mut();
            (self.replace)(&mut world, self.entity, new_value.clone());
            (old_value, new_value)
        };

        self.tween(old_value, new_value)
    }

    /// Creates a tween from an explicit starting value to a target value.
    pub fn animate_from<Object>(&self, from: T, to: T) -> Tween<Object> {
        self.animator.assert_timeline_mutation();
        let from = self.clamp(from);
        let to = self.clamp(to);
        let mut world = self.world.borrow_mut();
        (self.replace)(&mut world, self.entity, to.clone());
        drop(world);

        self.tween(from, to)
    }

    /// Creates a tween from the field's current value to a target value.
    pub fn animate<Object>(&self, to: T) -> Tween<Object> {
        self.animator.assert_timeline_mutation();
        let from = self.get();
        let to = self.clamp(to);
        let mut world = self.world.borrow_mut();
        (self.replace)(&mut world, self.entity, to.clone());
        drop(world);

        self.tween(from, to)
    }

    fn tween<Object>(&self, from: T, to: T) -> Tween<Object> {
        Tween::new(
            std::rc::Rc::clone(&self.world),
            self.entity,
            self.type_id,
            self.info,
            from.into_track_value(),
            to.into_track_value(),
            self.animator.clone(),
        )
    }

    fn clamp(&self, value: T) -> T {
        T::from_track_value(self.info.clamp(value.into_track_value()))
            .expect("Track limits must preserve the field value type.")
    }
}

impl TrackHandle<Quaternion> {
    /// Creates a local axis-angle rotation that preserves direction and winding.
    #[doc(hidden)]
    pub fn rotate_for<Object>(&self, axis: Vector3, angle: f32) -> Tween<Object> {
        self.animator.assert_timeline_mutation();
        let from = normalized_quaternion(self.get());
        assert!(
            axis.is_finite() && axis.length_squared() > f32::EPSILON,
            "Rotation axis must be finite and non-zero."
        );
        assert!(angle.is_finite(), "Rotation angle must be finite.");
        let axis = axis.normalize();
        let to = normalized_quaternion(from * Quaternion::from_axis_angle(axis, angle));

        {
            let mut world = self.world.borrow_mut();
            (self.replace)(&mut world, self.entity, to);
        }

        Tween::new_rotation(
            std::rc::Rc::clone(&self.world),
            self.entity,
            self.type_id,
            self.info,
            from,
            axis,
            angle,
            self.animator.clone(),
        )
    }
}

/// Describes a typed trackable property that can be animated on a compatible object.
#[derive(Clone, Copy)]
pub struct TrackProperty<T: TrackValueType> {
    type_id: std::any::TypeId,
    info: &'static TrackInfo,
    get: fn(&hecs::World, hecs::Entity) -> T,
    replace: fn(&mut hecs::World, hecs::Entity, T) -> T,
}

impl<T: TrackValueType> TrackProperty<T> {
    /// Creates metadata for a typed trackable property.
    pub const fn new(
        type_id: std::any::TypeId,
        info: &'static TrackInfo,
        get: fn(&hecs::World, hecs::Entity) -> T,
        replace: fn(&mut hecs::World, hecs::Entity, T) -> T,
    ) -> Self {
        Self {
            type_id,
            info,
            get,
            replace,
        }
    }

    pub(crate) const fn get_type_id(&self) -> std::any::TypeId {
        self.type_id
    }

    pub(crate) const fn get_info(&self) -> &'static TrackInfo {
        self.info
    }

    /// Creates a typed field handle for an object entity.
    pub fn handle(
        self,
        world: SceneWorld,
        entity: hecs::Entity,
        animator: AnimatorHandle,
    ) -> TrackHandle<T> {
        TrackHandle::new(
            world,
            entity,
            self.type_id,
            self.info,
            self.get,
            self.replace,
            animator,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, kinematic_macros::Trackable)]
    struct BoundedValues {
        #[track(min = 0.0, max = 1.0)]
        float: f32,
        #[track(min = -10)]
        signed: i32,
        #[track(max = 100)]
        unsigned: u32,
    }

    static TEST_TRACK_INFO: TrackInfo = TrackInfo {
        id: 0,
        name: "value",
        limits: TrackLimits::None,
        get: |_, _| TrackValue::F32(0.0),
        set: |_, _, _| {},
    };

    static BOUNDED_TRACK_INFO: TrackInfo = TrackInfo {
        id: 0,
        name: "value",
        limits: TrackLimits::F32 {
            min: Some(0.0),
            max: Some(1.0),
        },
        get: |world, entity| TrackValue::F32(*world.get::<&f32>(entity).unwrap()),
        set: |world, entity, value| {
            if let TrackValue::F32(value) = value {
                *world.get::<&mut f32>(entity).unwrap() = value;
            }
        },
    };

    #[test]
    fn generated_track_limits_clamp_every_numeric_type() {
        let mut world = hecs::World::new();
        let entity = world.spawn((BoundedValues {
            float: 0.5,
            signed: 0,
            unsigned: 50,
        },));

        (BoundedValues::track(0).set)(&world, entity, TrackValue::F32(2.0));
        (BoundedValues::track(1).set)(&world, entity, TrackValue::I32(-20));
        (BoundedValues::track(2).set)(&world, entity, TrackValue::U32(200));

        let values = world.get::<&BoundedValues>(entity).unwrap();
        assert_eq!(values.float, 1.0);
        assert_eq!(values.signed, -10);
        assert_eq!(values.unsigned, 100);
    }

    #[test]
    fn bounded_track_clamps_endpoints_and_interpolation_overshoot() {
        let mut track = Track::new(&BOUNDED_TRACK_INFO);
        track.add_tween(
            0.0,
            TrackValue::F32(-1.0),
            TrackValue::F32(2.0),
            1.0,
            Easing::OutBack,
        );

        assert_eq!(track.sample(0.0), Some(TrackValue::F32(0.0)));
        assert_eq!(track.sample(0.8), Some(TrackValue::F32(1.0)));
        assert_eq!(track.sample(1.0), Some(TrackValue::F32(1.0)));
    }

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
    fn instant_keyframe_preserves_the_tween_ending_at_the_same_time() {
        let mut track = Track::new(&TEST_TRACK_INFO);
        track.add_tween(
            0.0,
            TrackValue::F32(0.0),
            TrackValue::F32(1.0),
            1.0,
            Easing::Linear,
        );
        track.add_tween(
            1.0,
            TrackValue::F32(1.0),
            TrackValue::F32(0.0),
            0.0,
            Easing::Linear,
        );

        let (left, right) = track.find_keyframes(0.5);
        assert_eq!(left.unwrap().value, TrackValue::F32(0.0));
        assert_eq!(right.unwrap().value, TrackValue::F32(1.0));

        let coincident = track.keyframes_at(1.0);
        assert_eq!(coincident.len(), 2);
        assert_eq!(coincident[0].value, TrackValue::F32(1.0));
        assert_eq!(coincident[1].value, TrackValue::F32(0.0));

        let (left, right) = track.find_keyframes(1.0);
        assert_eq!(left.unwrap().value, TrackValue::F32(0.0));
        assert!(right.is_none());
    }

    #[test]
    fn vector3_tracks_lerp_and_convert() {
        let a = Vector3::new(2.0, 4.0, -2.0).into_track_value();
        let b = Vector3::new(10.0, -4.0, 6.0).into_track_value();
        assert_eq!(
            Vector3::from_track_value(a.lerp(&b, 0.25)),
            Some(Vector3::new(4.0, 2.0, 0.0))
        );
    }

    #[test]
    fn quaternion_tracks_slerp_normalize_and_take_the_short_path() {
        let a = Quaternion::IDENTITY;
        let b = Quaternion::from_rotation_y(std::f32::consts::FRAC_PI_2);
        for target in [b, -b, b * 2.0] {
            let value = a.into_track_value().lerp(&target.into_track_value(), 0.5);
            let q = Quaternion::from_track_value(value).unwrap();
            assert!((q.length() - 1.0).abs() < 1e-6);
            assert!(q.abs_diff_eq(
                Quaternion::from_rotation_y(std::f32::consts::FRAC_PI_4),
                1e-6
            ));
        }
        let same = b.into_track_value().lerp(&(-b).into_track_value(), 0.5);
        assert!(
            Quaternion::from_track_value(same)
                .unwrap()
                .abs_diff_eq(b, 1e-6)
        );
    }

    #[test]
    fn interpolates_vector_values() {
        let value = TrackValue::Vector2(Vector2::ZERO)
            .lerp(&TrackValue::Vector2(Vector2::new(10.0, 20.0)), 0.5);

        assert!(matches!(value, TrackValue::Vector2(vector) if vector == Vector2::new(5.0, 10.0)));
    }

    #[test]
    fn interpolates_quad_values() {
        let value = TrackValue::Quad(Quad::new(0.0, 10.0, 20.0, 30.0))
            .lerp(&TrackValue::Quad(Quad::new(10.0, 20.0, 30.0, 40.0)), 0.5);

        assert_eq!(value, TrackValue::Quad(Quad::new(5.0, 15.0, 25.0, 35.0)));
    }

    #[test]
    fn interpolates_color_values() {
        let value = TrackValue::Color(Color::new(0.0, 0.0, 0.0, 0.0))
            .lerp(&TrackValue::Color(Color::new(1.0, 0.5, 0.25, 1.0)), 0.5);

        assert_eq!(value, TrackValue::Color(Color::new(0.5, 0.25, 0.125, 0.5)));
    }

    #[test]
    fn string_values_change_at_the_segment_end() {
        let from = TrackValue::String("Source!".to_owned());
        let to = TrackValue::String("Aé🦀!".to_owned());

        assert_eq!(from.lerp(&to, 0.0), from);
        assert_eq!(from.lerp(&to, 0.5), from);
        assert_eq!(from.lerp(&to, 1.0), to);
    }

    #[test]
    fn converts_string_track_values() {
        let value = "Kinematic!".to_owned();
        let track_value = value.clone().into_track_value();

        assert_eq!(String::from_track_value(track_value), Some(value));
    }

    #[test]
    fn interpolates_bool_values_at_the_segment_end() {
        let from = TrackValue::Bool(false);
        let to = TrackValue::Bool(true);

        assert_eq!(from.lerp(&to, 0.0), from);
        assert_eq!(from.lerp(&to, 0.5), from);
        assert_eq!(from.lerp(&to, 0.999), from);
        assert_eq!(from.lerp(&to, 1.0), to);
    }

    #[test]
    fn converts_bool_track_values() {
        let track_value = true.into_track_value();

        assert_eq!(bool::from_track_value(track_value), Some(true));
    }

    #[test]
    fn interpolates_u32_values_to_the_nearest_integer() {
        let from = TrackValue::U32(10);
        let to = TrackValue::U32(20);

        assert_eq!(from.lerp(&to, 0.55), TrackValue::U32(16));
        assert_eq!(from.lerp(&to, 1.0), to);
    }

    #[test]
    fn converts_u32_track_values() {
        let track_value = 42_u32.into_track_value();

        assert_eq!(u32::from_track_value(track_value), Some(42));
    }

    #[test]
    fn interpolates_i32_values_to_the_nearest_integer() {
        let from = TrackValue::I32(-10);
        let to = TrackValue::I32(10);

        assert_eq!(from.lerp(&to, 0.25), TrackValue::I32(-5));
        assert_eq!(from.lerp(&to, 0.53), TrackValue::I32(1));
        assert_eq!(from.lerp(&to, 1.0), to);
    }

    #[test]
    fn converts_i32_track_values() {
        let track_value = (-42_i32).into_track_value();

        assert_eq!(i32::from_track_value(track_value), Some(-42));
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

    #[test]
    fn sample_evaluates_without_writing_to_the_world() {
        let track = tween_track();
        let mut world = hecs::World::new();
        let entity = world.spawn((0.0_f32,));

        assert_eq!(track.sample(5.0), Some(TrackValue::F32(5.0)));
        assert_eq!(*world.get::<&f32>(entity).unwrap(), 0.0);
    }
}

#[derive(Debug)]
pub struct TrackInfo {
    /// Stable track id inside the component type.
    pub id: TrackId,
    /// Human-readable field name used by tooling and debugging.
    pub name: &'static str,
    /// Optional numeric bounds shared by animation and editing.
    pub limits: TrackLimits,
    /// Reads the current value of the tracked field.
    pub get: TrackGetter,
    /// Writes an interpolated value back to the field.
    pub set: TrackSetter,
}

impl TrackInfo {
    /// Restricts a runtime value to this track's numeric bounds.
    pub fn clamp(&self, value: TrackValue) -> TrackValue {
        match (value, self.limits) {
            (TrackValue::F32(value), TrackLimits::F32 { min, max }) => {
                TrackValue::F32(clamp_numeric(value, min, max))
            }
            (TrackValue::I32(value), TrackLimits::I32 { min, max }) => {
                TrackValue::I32(clamp_numeric(value, min, max))
            }
            (TrackValue::U32(value), TrackLimits::U32 { min, max }) => {
                TrackValue::U32(clamp_numeric(value, min, max))
            }
            (value, _) => value,
        }
    }
}

fn clamp_numeric<T: PartialOrd>(mut value: T, min: Option<T>, max: Option<T>) -> T {
    if let Some(min) = min
        && value < min
    {
        value = min;
    }
    if let Some(max) = max
        && value > max
    {
        value = max;
    }
    value
}

/// Optional bounds for a numeric track.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrackLimits {
    /// An unbounded track.
    None,
    /// Bounds for an `f32` track.
    F32 { min: Option<f32>, max: Option<f32> },
    /// Bounds for an `i32` track.
    I32 { min: Option<i32>, max: Option<i32> },
    /// Bounds for a `u32` track.
    U32 { min: Option<u32>, max: Option<u32> },
}

/// Metadata for a component that exposes one or more tracked fields.
#[derive(Clone, Copy)]
pub struct TrackableInfo {
    /// Name of the trackable component, usually the Rust type name.
    pub name: &'static str,
    /// Returns the runtime type id of the trackable component.
    pub type_id: fn() -> std::any::TypeId,
    /// Returns the static list of tracked fields for the component.
    pub get: fn() -> &'static [TrackInfo],
}

/// Trait implemented by types that expose animatable fields.
pub trait Trackable {
    /// Internal field layer added to generated object handlers.
    type HandlerFields<Next: HandlerContext>;

    /// Builds this component's tracked fields around the next handler layer.
    fn handler_fields<Next: HandlerContext>(
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

/// Interprets invalid rotations as identity and normalizes valid rotations.
pub(crate) fn normalized_quaternion(value: Quaternion) -> Quaternion {
    if value.is_finite() && value.length_squared() > f32::EPSILON {
        value.normalize()
    } else {
        Quaternion::IDENTITY
    }
}
