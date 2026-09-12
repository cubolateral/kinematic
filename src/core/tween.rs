use crate::core::{
    AnimatorHandle, Easing, SceneWorld, Task, TrackInfo, TrackProperty, TrackValue, TrackValueType,
    normalized_quaternion,
    types::{Quaternion, Vector3},
};

type PrepareTween<Object> = Box<dyn FnOnce(&Tween<Object>)>;

struct TweenTarget {
    type_id: std::any::TypeId,
    track_info: &'static TrackInfo,
    from: TrackValue,
    to: TrackValue,
    rotation: Option<RotationTarget>,
}

struct RotationTarget {
    from: Quaternion,
    axis: Vector3,
    angle: f32,
}

/// Describes simultaneous interpolation of one or more tracked fields.
///
/// A tween becomes part of its scene timeline when [`Self::play`] is called.
/// Additional fields from the same object can be chained before setting the
/// shared duration and easing. It can also be converted into a [`Task`] and
/// passed to [`Scene::play`](crate::core::Scene::play) manually.
pub struct Tween<Object = ()> {
    world: SceneWorld,
    entity: hecs::Entity,
    targets: Vec<TweenTarget>,
    prepare: Option<PrepareTween<Object>>,
    delay: f32,
    duration: f32,
    easing: Easing,
    animator: AnimatorHandle,
    object: std::marker::PhantomData<Object>,
}

impl<Object> Tween<Object> {
    pub(crate) fn context(&self) -> (SceneWorld, AnimatorHandle) {
        (std::rc::Rc::clone(&self.world), self.animator.active())
    }

    // Prepare derived data after all chained properties have been supplied.
    pub(crate) fn prepare(mut self, prepare: impl FnOnce(&Self) + 'static) -> Self {
        self.prepare = Some(Box::new(prepare));
        self
    }

    // Evaluate a component endpoint in isolation, without changing scene state.
    pub(crate) fn endpoint<C: hecs::Component + Clone>(&self, base: &C, end: bool) -> C {
        let mut snapshot = hecs::World::new();
        let entity = snapshot.spawn((base.clone(),));
        for target in &self.targets {
            if target.type_id == std::any::TypeId::of::<C>() {
                let value = if end { &target.to } else { &target.from };
                (target.track_info.set)(&snapshot, entity, value.clone());
            }
        }
        snapshot.remove_one::<C>(entity).unwrap()
    }

    /// Creates a tween with a one-second duration and [`Easing::default()`] easing.
    pub fn new(
        world: SceneWorld,
        entity: hecs::Entity,
        type_id: std::any::TypeId,
        track_info: &'static TrackInfo,
        from: TrackValue,
        to: TrackValue,
        animator: AnimatorHandle,
    ) -> Self {
        Self {
            world,
            entity,
            targets: vec![TweenTarget {
                type_id,
                track_info,
                from,
                to,
                rotation: None,
            }],
            prepare: None,
            delay: 0.0,
            duration: 1.0,
            easing: Easing::default(),
            animator,
            object: std::marker::PhantomData,
        }
    }

    /// Creates a tween containing an arbitrary set of simultaneous targets.
    pub(crate) fn from_targets(
        world: SceneWorld,
        entity: hecs::Entity,
        targets: Vec<(std::any::TypeId, &'static TrackInfo, TrackValue, TrackValue)>,
        animator: AnimatorHandle,
    ) -> Self {
        Self {
            world,
            entity,
            targets: targets
                .into_iter()
                .map(|(type_id, track_info, from, to)| TweenTarget {
                    type_id,
                    track_info,
                    from,
                    to,
                    rotation: None,
                })
                .collect(),
            prepare: None,
            delay: 0.0,
            duration: 1.0,
            easing: Easing::default(),
            animator,
            object: std::marker::PhantomData,
        }
    }

    /// Adds or replaces a target field in this simultaneous tween.
    #[doc(hidden)]
    pub fn set_track<T: TrackValueType>(
        self,
        type_id: std::any::TypeId,
        track_info: &'static TrackInfo,
        value: T,
    ) -> Self {
        self.update_track(type_id, track_info, |_| value)
    }

    /// Updates a target field while preserving the tween's original value.
    #[doc(hidden)]
    pub fn update_track<T: TrackValueType>(
        mut self,
        type_id: std::any::TypeId,
        track_info: &'static TrackInfo,
        update: impl FnOnce(T) -> T,
    ) -> Self {
        let (from, to) = {
            let world = self.world.borrow();
            let from = (track_info.get)(&world, self.entity);
            let value = T::from_track_value(from.clone())
                .expect("Track metadata must return its declared value type.");
            let to = update(value).into_track_value();

            (track_info.set)(&world, self.entity, to.clone());
            (from, to)
        };

        if let Some(target) = self
            .targets
            .iter_mut()
            .find(|target| target.type_id == type_id && std::ptr::eq(target.track_info, track_info))
        {
            target.to = to;
            target.rotation = None;
        } else {
            self.targets.push(TweenTarget {
                type_id,
                track_info,
                from,
                to,
                rotation: None,
            });
        }

        self
    }

    /// Adds or replaces a typed property with an explicit starting value.
    pub fn animate_from<T: TrackValueType>(
        mut self,
        property: TrackProperty<T>,
        from: T,
        to: T,
    ) -> Self {
        let type_id = property.get_type_id();
        let track_info = property.get_info();
        let from = from.into_track_value();
        let to = to.into_track_value();

        {
            let world = self.world.borrow();
            (track_info.set)(&world, self.entity, to.clone());
        }

        if let Some(target) = self
            .targets
            .iter_mut()
            .find(|target| target.type_id == type_id && std::ptr::eq(target.track_info, track_info))
        {
            target.from = from;
            target.to = to;
            target.rotation = None;
        } else {
            self.targets.push(TweenTarget {
                type_id,
                track_info,
                from,
                to,
                rotation: None,
            });
        }

        self
    }

    /// Sets the duration in timeline seconds for every target field.
    pub fn duration(mut self, duration: f32) -> Self {
        self.duration = duration;
        self
    }

    /// Delays this tween by the specified number of timeline seconds.
    pub fn delay(mut self, duration: f32) -> Self {
        self.delay = duration;
        self
    }

    /// Sets the easing function used by every target field.
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Adds or replaces an axis-angle quaternion target.
    #[doc(hidden)]
    pub fn rotate_track(
        mut self,
        property: TrackProperty<Quaternion>,
        axis: Vector3,
        angle: f32,
    ) -> Self {
        validate_rotation(axis, angle);
        let axis = axis.normalize();
        let type_id = property.get_type_id();
        let track_info = property.get_info();
        let from = normalized_quaternion(
            property
                .handle(
                    std::rc::Rc::clone(&self.world),
                    self.entity,
                    self.animator.clone(),
                )
                .get(),
        );
        let to = normalized_quaternion(from * Quaternion::from_axis_angle(axis, angle));

        {
            let world = self.world.borrow();
            (track_info.set)(&world, self.entity, TrackValue::Quaternion(to));
        }

        if let Some(target) = self
            .targets
            .iter_mut()
            .find(|target| target.type_id == type_id && std::ptr::eq(target.track_info, track_info))
        {
            target.from = TrackValue::Quaternion(from);
            target.to = TrackValue::Quaternion(to);
            target.rotation = Some(RotationTarget { from, axis, angle });
        } else {
            self.targets.push(TweenTarget {
                type_id,
                track_info,
                from: TrackValue::Quaternion(from),
                to: TrackValue::Quaternion(to),
                rotation: Some(RotationTarget { from, axis, angle }),
            });
        }

        self
    }

    /// Immediately plays this tween as a shortcut for `.duration(0.0).play()`.
    pub fn immediate(self) {
        self.duration(0.0).play();
    }

    /// Registers this tween in the animator associated with its object handler.
    pub fn play(self) {
        let animator = self.animator.active();
        animator.play(self.task());
    }

    /// Converts this description into one task that runs all target fields together.
    pub fn task(mut self) -> Task {
        if let Some(prepare) = self.prepare.take() {
            prepare(&self);
        }
        let mut tasks: Vec<_> = self
            .targets
            .into_iter()
            .map(|target| match target.rotation {
                Some(rotation) => Task::RotationTween {
                    entity: self.entity,
                    type_id: target.type_id,
                    track_info: target.track_info,
                    from: rotation.from,
                    axis: rotation.axis,
                    angle: rotation.angle,
                    duration: self.duration,
                    easing: self.easing,
                },
                None => Task::Tween {
                    entity: self.entity,
                    type_id: target.type_id,
                    track_info: target.track_info,
                    from: target.from,
                    to: target.to,
                    duration: self.duration,
                    easing: self.easing,
                },
            })
            .collect();

        let task = if tasks.len() == 1 {
            tasks.pop().unwrap()
        } else {
            Task::All(tasks)
        };

        if self.delay == 0.0 {
            task
        } else {
            task.delay(self.delay)
        }
    }
}

impl Tween<()> {
    pub(crate) fn new_rotation<Object>(
        world: SceneWorld,
        entity: hecs::Entity,
        type_id: std::any::TypeId,
        track_info: &'static TrackInfo,
        from: Quaternion,
        axis: Vector3,
        angle: f32,
        animator: AnimatorHandle,
    ) -> Tween<Object> {
        validate_rotation(axis, angle);
        let axis = axis.normalize();
        let from = normalized_quaternion(from);
        let to = normalized_quaternion(from * Quaternion::from_axis_angle(axis, angle));

        Tween {
            world,
            entity,
            targets: vec![TweenTarget {
                type_id,
                track_info,
                from: TrackValue::Quaternion(from),
                to: TrackValue::Quaternion(to),
                rotation: Some(RotationTarget { from, axis, angle }),
            }],
            prepare: None,
            delay: 0.0,
            duration: 1.0,
            easing: Easing::default(),
            animator,
            object: std::marker::PhantomData,
        }
    }
}

fn validate_rotation(axis: Vector3, angle: f32) {
    assert!(
        axis.is_finite() && axis.length_squared() > f32::EPSILON,
        "Rotation axis must be finite and non-zero."
    );
    assert!(angle.is_finite(), "Rotation angle must be finite.");
}

#[cfg(test)]
mod tests {
    use crate::core::components::Style;
    use crate::prelude::*;

    #[test]
    fn endpoints_include_overrides_without_mutating_the_scene() {
        let mut scene = Scene::new();
        let object = text_2d().fill(Color::RED).build(&mut scene);
        let tween = object.fill(Color::YELLOW).fill(Color::BLUE).animate_from(
            Style::stroke_width_property(),
            2.0,
            8.0,
        );
        let from = tween.endpoint(&Style::default(), false);
        let to = tween.endpoint(&Style::default(), true);
        assert_eq!(from.fill, Color::RED);
        assert_eq!(to.fill, Color::BLUE);
        assert_eq!(from.stroke_width, 2.0);
        assert_eq!(to.stroke_width, 8.0);
        assert_eq!(object.get(Style::fill_property()), Color::BLUE);
        assert_eq!(object.get(Style::stroke_width_property()), 8.0);
    }
}
