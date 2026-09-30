use crate::core::{
    Easing, Scene, SignalContext, SignalFrame, SignalHandle, Task, TrackInfo, TrackTarget,
    TrackValue,
    components::Animation,
    normalized_quaternion,
    track::TrackRepeat,
    types::{Quaternion, Vector3},
};

#[doc(hidden)]
#[derive(Clone)]
pub struct AnimatorHandle {
    state: std::rc::Rc<std::cell::RefCell<AnimatorState>>,
    context: std::rc::Rc<AnimatorContext>,
}

struct AnimatorContext {
    time: std::rc::Rc<std::cell::Cell<f32>>,
    active: std::cell::RefCell<std::rc::Weak<std::cell::RefCell<AnimatorState>>>,
    signals: std::rc::Rc<SignalContext>,
}

struct AnimatorState {
    schedule: Schedule,
    start_time: f32,
    scheduling: Scheduling,
    repeating: bool,
}

impl AnimatorState {
    fn time(&self) -> f32 {
        self.start_time + self.scheduling.offset(self.schedule.duration)
    }
}

pub(crate) struct Animator {
    handle: AnimatorHandle,
}

#[derive(Clone, Copy)]
pub(crate) enum Scheduling {
    Sequential,
    Parallel,
}

impl Scheduling {
    fn offset(self, duration: f32) -> f32 {
        match self {
            Self::Sequential => duration,
            Self::Parallel => 0.0,
        }
    }
}

/// A normalized timeline with no sequential or parallel task nodes.
#[derive(Default)]
pub(crate) struct Schedule {
    pub(crate) duration: f32,
    pub(crate) parallel: bool,
    tweens: Vec<ScheduledTween>,
    repeats: Vec<ScheduledRepeat>,
}

struct ScheduledRepeat {
    start: f32,
    duration: f32,
    tweens: Vec<ScheduledTween>,
}

struct ScheduledTween {
    start: f32,
    entity: hecs::Entity,
    target: TrackTarget,
    from: TrackValue,
    to: TrackValue,
    duration: f32,
    easing: Easing,
    rotation: Option<(Vector3, f32)>,
    mask: u8,
    relative: u8,
    implicit: bool,
}

impl ScheduledTween {
    fn redundant(
        &self,
        animation: &Animation,
        world: &hecs::World,
        later: &[Self],
        repeats: &[ScheduledRepeat],
    ) -> bool {
        if self.mask == 0 {
            return true;
        }
        if self.rotation.is_some() || self.from != self.to || self.mask.count_ones() != 1 {
            return false;
        }

        let channel = self.mask.trailing_zeros() as u8;
        let target = if self.from.channels() == 1 {
            self.target.clone()
        } else {
            self.target.clone().component(channel)
        };
        let same_value = |left: &TrackValue, right: &TrackValue| {
            if self.from.channels() == 1 {
                left == right
            } else {
                left.channel(channel) == right.channel(channel)
            }
        };
        let next = later.iter().find(|tween| {
            tween.entity == self.entity
                && tween.target.same_field(&self.target)
                && tween.mask & self.mask != 0
        });
        if next.is_some_and(|next| next.start < self.start + self.duration)
            || repeats.iter().any(|repeat| {
                repeat.tweens.iter().any(|tween| {
                    tween.entity == self.entity
                        && tween.target.same_field(&self.target)
                        && tween.mask & self.mask != 0
                })
            })
        {
            return false;
        }

        if let Some(track) = animation
            .tracks
            .iter()
            .find(|track| track.track.target.same(&target))
        {
            return track.track.repeat.is_none()
                && track.track.keyframes.last().is_some_and(|last| {
                    last.time <= self.start && same_value(&last.value, &self.from)
                });
        }

        next.map_or_else(
            || same_value(&target.get(world, self.entity), &self.from),
            |next| same_value(&next.from, &self.from),
        )
    }

    fn append(self, animation: &mut Animation, offset: f32) {
        let start = offset + self.start;
        for channel in 0..self.from.channels() {
            let bit = 1 << channel;
            if self.mask & bit == 0 {
                continue;
            }
            let target = if self.from.channels() == 1 {
                self.target.clone()
            } else {
                self.target.clone().component(channel)
            };
            let track = animation.target_mut(target);
            let from = if self.implicit {
                track.sample(start).unwrap_or_else(|| self.from.clone())
            } else {
                self.from.clone()
            };
            let to = if self.relative & bit != 0 {
                from.add_difference(&self.from, &self.to)
            } else {
                self.to.clone()
            };
            if let Some((axis, angle)) = self.rotation {
                let TrackValue::Quaternion(from) = from else {
                    unreachable!("Rotation must have a quaternion starting value.");
                };
                track.add_rotation_tween(start, from, axis, angle, self.duration, self.easing);
            } else {
                track.add_tween(start, from, to, self.duration, self.easing);
            }
        }
    }
}

impl Schedule {
    fn from_task(task: Task) -> Self {
        match task {
            Task::Tween {
                entity,
                type_id,
                track_info,
                from,
                to,
                duration,
                easing,
            } => Self::tween(ScheduledTween {
                start: 0.0,
                entity,
                target: TrackTarget::property(type_id, track_info),
                mask: (1 << from.channels()) - 1,
                from,
                to,
                duration,
                easing,
                rotation: None,
                relative: 0,
                implicit: false,
            }),
            Task::PropertyTween {
                entity,
                type_id,
                track_info,
                from,
                to,
                duration,
                easing,
                mask,
                relative,
                implicit,
            } => Self::tween(ScheduledTween {
                start: 0.0,
                entity,
                target: TrackTarget::property(type_id, track_info),
                from,
                to,
                duration,
                easing,
                rotation: None,
                mask,
                relative,
                implicit,
            }),
            Task::UniformTween {
                entity,
                name,
                from,
                to,
                duration,
                easing,
            } => Self::tween(ScheduledTween {
                start: 0.0,
                entity,
                target: TrackTarget::uniform(name),
                mask: (1 << from.channels()) - 1,
                from,
                to,
                duration,
                easing,
                rotation: None,
                relative: 0,
                implicit: false,
            }),
            Task::UniformPropertyTween {
                entity,
                name,
                from,
                to,
                duration,
                easing,
                implicit,
            } => Self::tween(ScheduledTween {
                start: 0.0,
                entity,
                target: TrackTarget::uniform(name),
                mask: if implicit {
                    from.changed_channels(&to)
                } else {
                    (1 << from.channels()) - 1
                },
                from,
                to,
                duration,
                easing,
                rotation: None,
                relative: 0,
                implicit,
            }),
            Task::RotationTween {
                entity,
                type_id,
                track_info,
                from,
                axis,
                angle,
                duration,
                easing,
            } => {
                assert!(
                    axis.is_finite() && axis.length_squared() > f32::EPSILON,
                    "Rotation axis must be finite and non-zero."
                );
                assert!(angle.is_finite(), "Rotation angle must be finite.");
                let axis = axis.normalize();
                let from = normalized_quaternion(from);
                let to = normalized_quaternion(from * Quaternion::from_axis_angle(axis, angle));
                Self::tween(ScheduledTween {
                    start: 0.0,
                    entity,
                    target: TrackTarget::property(type_id, track_info),
                    from: TrackValue::Quaternion(from),
                    to: TrackValue::Quaternion(to),
                    duration,
                    easing,
                    rotation: Some((axis, angle)),
                    mask: 1,
                    relative: 0,
                    implicit: false,
                })
            }
            Task::RotationPropertyTween {
                entity,
                type_id,
                track_info,
                from,
                axis,
                angle,
                duration,
                easing,
            } => {
                assert!(
                    axis.is_finite() && axis.length_squared() > f32::EPSILON,
                    "Rotation axis must be finite and non-zero."
                );
                assert!(angle.is_finite(), "Rotation angle must be finite.");
                let axis = axis.normalize();
                let from = normalized_quaternion(from);
                let to = normalized_quaternion(from * Quaternion::from_axis_angle(axis, angle));
                Self::tween(ScheduledTween {
                    start: 0.0,
                    entity,
                    target: TrackTarget::property(type_id, track_info),
                    from: TrackValue::Quaternion(from),
                    to: TrackValue::Quaternion(to),
                    duration,
                    easing,
                    rotation: Some((axis, angle)),
                    mask: 1,
                    relative: 0,
                    implicit: true,
                })
            }
            Task::Wait(duration) => {
                validate_duration(duration);
                Self {
                    duration,
                    ..Self::default()
                }
            }
            Task::Chain(tasks) => Self::group(tasks, Scheduling::Sequential),
            Task::All(tasks) => Self::group(tasks, Scheduling::Parallel),
            Task::Repeat(tasks) => Self::group(tasks, Scheduling::Sequential).repeated(),
        }
    }

    fn tween(tween: ScheduledTween) -> Self {
        validate_duration(tween.duration);
        Self {
            duration: tween.duration,
            tweens: vec![tween],
            ..Self::default()
        }
    }

    fn group(tasks: Vec<Task>, scheduling: Scheduling) -> Self {
        let mut result = Self::default();
        for task in tasks {
            result.append(Self::from_task(task), scheduling);
        }
        result
    }

    fn append(&mut self, mut child: Self, scheduling: Scheduling) {
        let offset = scheduling.offset(self.duration);
        if !child.parallel {
            self.duration = self.duration.max(offset + child.duration);
        }
        validate_duration(self.duration);
        for tween in &mut child.tweens {
            tween.start += offset;
        }
        for repeat in &mut child.repeats {
            repeat.start += offset;
        }
        self.tweens.extend(child.tweens);
        self.repeats.extend(child.repeats);
    }

    pub(crate) fn repeated(self) -> Self {
        assert!(
            self.repeats.is_empty(),
            "Repeat cycles cannot contain another repeat."
        );
        assert!(
            self.duration.is_finite() && self.duration > 0.0,
            "Repeat cycles must have a finite, positive duration."
        );
        assert!(
            !self.tweens.is_empty(),
            "Repeat cycles must contain an animation."
        );
        Self {
            duration: 0.0,
            parallel: false,
            tweens: vec![],
            repeats: vec![ScheduledRepeat {
                start: 0.0,
                duration: self.duration,
                tweens: self
                    .tweens
                    .into_iter()
                    .map(|mut tween| {
                        if tween.mask == 0 {
                            tween.mask = (1 << tween.from.channels()) - 1;
                        }
                        tween
                    })
                    .collect(),
            }],
        }
    }

    pub(crate) fn compile(mut self, scene: &Scene) -> f32 {
        // Stable ordering preserves instantaneous changes at shared endpoints.
        self.tweens.sort_by(|a, b| a.start.total_cmp(&b.start));
        let world = scene.world();
        let mut tweens = self.tweens.into_iter();
        while let Some(tween) = tweens.next() {
            let mut animation = world.get::<&mut Animation>(tween.entity).unwrap();
            if !tween.redundant(&animation, &world, tweens.as_slice(), &self.repeats) {
                tween.append(&mut animation, 0.0);
            }
        }
        for mut repeat in self.repeats {
            repeat.tweens.sort_by(|a, b| a.start.total_cmp(&b.start));
            let mut initialized: Vec<(hecs::Entity, TrackTarget)> = Vec::new();
            for tween in repeat.tweens {
                let mut animation = world.get::<&mut Animation>(tween.entity).unwrap();
                for channel in 0..tween.from.channels() {
                    if tween.mask & (1 << channel) == 0 {
                        continue;
                    }
                    let target = if tween.from.channels() == 1 {
                        tween.target.clone()
                    } else {
                        tween.target.clone().component(channel)
                    };
                    if initialized.iter().any(|(entity, initialized)| {
                        *entity == tween.entity && initialized.same(&target)
                    }) {
                        continue;
                    }
                    initialized.push((tween.entity, target.clone()));
                    let name = target.name().to_owned();
                    let track = animation.target_mut(target);
                    assert!(
                        track.repeat.is_none()
                            && track
                                .keyframes
                                .last()
                                .is_none_or(|last| last.time <= repeat.start),
                        "Repeat overlaps another animation on property '{}'.",
                        name
                    );
                    track.add_tween(
                        repeat.start,
                        tween.from.clone(),
                        tween.from.clone(),
                        0.0,
                        Easing::Linear,
                    );
                    track.repeat = Some(TrackRepeat {
                        start: repeat.start,
                        duration: repeat.duration,
                    });
                }
                tween.append(&mut animation, repeat.start);
            }
        }
        drop(world);
        scene.compile_runtime();
        self.duration
    }
}

fn validate_duration(duration: f32) {
    assert!(
        duration.is_finite() && duration >= 0.0,
        "Animation durations must be finite and non-negative."
    );
}

impl Animator {
    pub(crate) fn with_scene_time(scene_time: std::rc::Rc<std::cell::Cell<f32>>) -> Self {
        scene_time.set(0.0);
        let state = std::rc::Rc::new(std::cell::RefCell::new(AnimatorState {
            schedule: Schedule::default(),
            start_time: 0.0,
            scheduling: Scheduling::Sequential,
            repeating: false,
        }));
        let context = std::rc::Rc::new(AnimatorContext {
            time: scene_time,
            active: std::cell::RefCell::new(std::rc::Rc::downgrade(&state)),
            signals: std::rc::Rc::new(SignalContext::default()),
        });
        Self {
            handle: AnimatorHandle { state, context },
        }
    }

    pub(crate) fn handle(&self) -> AnimatorHandle {
        self.handle.clone()
    }

    pub(crate) fn group(&self, scheduling: Scheduling, repeating: bool) -> Self {
        let state = self.handle.state.borrow();
        assert!(
            !repeating || !state.repeating,
            "Repeat cycles cannot contain another repeat."
        );
        Self {
            handle: AnimatorHandle {
                state: std::rc::Rc::new(std::cell::RefCell::new(AnimatorState {
                    schedule: Schedule::default(),
                    start_time: state.time(),
                    scheduling,
                    repeating: state.repeating || repeating,
                })),
                context: std::rc::Rc::clone(&self.handle.context),
            },
        }
    }

    pub(crate) fn take_schedule(&self) -> Schedule {
        let mut state = self.handle.state.borrow_mut();
        let duration = state.schedule.duration;
        std::mem::replace(
            &mut state.schedule,
            Schedule {
                duration,
                ..Schedule::default()
            },
        )
    }

    pub(crate) fn duration(&self) -> f32 {
        self.handle.state.borrow().schedule.duration
    }
}

impl AnimatorHandle {
    pub(crate) fn activate(&self) -> Self {
        let previous = self.active();
        self.context
            .active
            .replace(std::rc::Rc::downgrade(&self.state));
        self.sync_scene_time();
        previous
    }

    pub(crate) fn restore(&self, previous: Self) {
        previous.activate();
    }

    pub(crate) fn active(&self) -> Self {
        Self {
            state: self
                .context
                .active
                .borrow()
                .upgrade()
                .unwrap_or_else(|| self.state.clone()),
            context: self.context.clone(),
        }
    }

    #[doc(hidden)]
    pub fn time(&self) -> f32 {
        self.context.time.get()
    }

    /// Rejects scene mutations that cannot be replayed as a periodic animation.
    #[doc(hidden)]
    pub fn assert_finite_scope(&self) {
        self.assert_timeline_mutation();
        assert!(
            !self.active().state.borrow().repeating,
            "Repeat cycles can only schedule animations and waits; create, attach, or remove objects outside repeat."
        );
    }

    /// Rejects timeline and scene-structure changes while a signal is running.
    #[doc(hidden)]
    pub fn assert_timeline_mutation(&self) {
        assert!(
            !self.context.signals.is_evaluating(),
            "Signals cannot alter the scene structure or timeline while they are evaluated."
        );
    }

    pub(crate) fn assert_event_scope(&self) {
        self.assert_timeline_mutation();
        assert!(
            !self.active().state.borrow().repeating,
            "Events cannot be used inside repeat cycles."
        );
    }

    pub(crate) fn signal(
        &self,
        target: hecs::Entity,
        callback: impl FnMut(SignalFrame) + 'static,
    ) -> SignalHandle {
        let active = self.active();
        active.assert_finite_scope();
        let start = active.state.borrow().time();
        self.context.signals.add(target, start, callback, active)
    }

    pub(crate) fn signals(&self) -> std::rc::Rc<SignalContext> {
        std::rc::Rc::clone(&self.context.signals)
    }

    pub(crate) fn record_signal_override(
        &self,
        entity: hecs::Entity,
        type_id: std::any::TypeId,
        track_info: &'static TrackInfo,
        value: TrackValue,
    ) {
        self.context
            .signals
            .record_override(entity, type_id, track_info, value);
    }

    pub(crate) fn record_signal_uniform_override(
        &self,
        entity: hecs::Entity,
        name: impl Into<String>,
        value: TrackValue,
    ) {
        self.context
            .signals
            .record_uniform_override(entity, name, value);
    }

    fn sync_scene_time(&self) {
        self.context.time.set(self.state.borrow().time());
    }

    pub(crate) fn play(&self, task: Task) {
        self.schedule(Schedule::from_task(task));
    }

    pub(crate) fn schedule(&self, schedule: Schedule) {
        self.assert_timeline_mutation();
        let active = self.active();
        let mut state = active.state.borrow_mut();
        assert!(
            !state.repeating || schedule.repeats.is_empty(),
            "Repeat cycles cannot contain another repeat."
        );
        let scheduling = state.scheduling;
        state.schedule.append(schedule, scheduling);
        self.context.time.set(state.time());
    }
}
