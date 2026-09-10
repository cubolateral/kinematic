use crate::core::{
    Easing, Scene, SignalContext, SignalHandle, Task, TrackInfo, TrackValue,
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
    type_id: std::any::TypeId,
    track_info: &'static TrackInfo,
    from: TrackValue,
    to: TrackValue,
    duration: f32,
    easing: Easing,
    rotation: Option<(Vector3, f32)>,
}

impl ScheduledTween {
    fn append(self, animation: &mut Animation, offset: f32) {
        let track = animation.track_mut(self.type_id, self.track_info);
        let start = offset + self.start;
        assert!(
            track.keyframes.last().is_none_or(|last| last.time <= start),
            "Animations on property '{}' overlap.",
            self.track_info.name,
        );
        if let Some((axis, angle)) = self.rotation {
            let TrackValue::Quaternion(from) = self.from else {
                unreachable!("Rotation must have a quaternion starting value.");
            };
            track.add_rotation_tween(start, from, axis, angle, self.duration, self.easing);
        } else {
            track.add_tween(start, self.from, self.to, self.duration, self.easing);
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
                type_id,
                track_info,
                from,
                to,
                duration,
                easing,
                rotation: None,
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
                    type_id,
                    track_info,
                    from: TrackValue::Quaternion(from),
                    to: TrackValue::Quaternion(to),
                    duration,
                    easing,
                    rotation: Some((axis, angle)),
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
            repeats: vec![],
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
        self.duration = self.duration.max(offset + child.duration);
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
            tweens: vec![],
            repeats: vec![ScheduledRepeat {
                start: 0.0,
                duration: self.duration,
                tweens: self.tweens,
            }],
        }
    }

    pub(crate) fn compile(mut self, scene: &Scene) -> f32 {
        // Stable ordering preserves instantaneous changes at shared endpoints.
        self.tweens.sort_by(|a, b| a.start.total_cmp(&b.start));
        let world = scene.get_world();
        for tween in self.tweens {
            let mut animation = world.get::<&mut Animation>(tween.entity).unwrap();
            tween.append(&mut animation, 0.0);
        }
        for mut repeat in self.repeats {
            repeat.tweens.sort_by(|a, b| a.start.total_cmp(&b.start));
            let mut initialized = std::collections::HashSet::new();
            for tween in repeat.tweens {
                let mut animation = world.get::<&mut Animation>(tween.entity).unwrap();
                if initialized.insert((tween.entity, tween.type_id, tween.track_info.id)) {
                    let track = animation.track_mut(tween.type_id, tween.track_info);
                    assert!(
                        track.repeat.is_none()
                            && track
                                .keyframes
                                .last()
                                .is_none_or(|last| last.time <= repeat.start),
                        "Repeat overlaps another animation on property '{}'.",
                        tween.track_info.name
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

    pub(crate) fn signal(
        &self,
        target: hecs::Entity,
        callback: impl FnMut() + 'static,
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
