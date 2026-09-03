use crate::core::{
    AnimatorHandle, Easing, SceneWorld, Task, TrackInfo, TrackProperty, TrackValue, TrackValueType,
};

struct TweenTarget {
    type_id: std::any::TypeId,
    track_info: &'static TrackInfo,
    from: TrackValue,
    to: TrackValue,
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
    duration: f32,
    easing: Easing,
    animator: AnimatorHandle,
    object: std::marker::PhantomData<Object>,
}

impl<Object> Tween<Object> {
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
            }],
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
                })
                .collect(),
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
        } else {
            self.targets.push(TweenTarget {
                type_id,
                track_info,
                from,
                to,
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
        } else {
            self.targets.push(TweenTarget {
                type_id,
                track_info,
                from,
                to,
            });
        }

        self
    }

    /// Sets the duration in timeline seconds for every target field.
    pub fn duration(mut self, duration: f32) -> Self {
        self.duration = duration;
        self
    }

    /// Sets the easing function used by every target field.
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
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
    pub fn task(self) -> Task {
        let mut tasks: Vec<_> = self
            .targets
            .into_iter()
            .map(|target| Task::Tween {
                entity: self.entity,
                type_id: target.type_id,
                track_info: target.track_info,
                from: target.from,
                to: target.to,
                duration: self.duration,
                easing: self.easing,
            })
            .collect();

        if tasks.len() == 1 {
            tasks.pop().unwrap()
        } else {
            Task::All(tasks)
        }
    }
}
