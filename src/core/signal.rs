use crate::core::{AnimatorHandle, SceneWorld, TrackInfo, TrackValue, components::Node};

type SignalCallback = std::rc::Rc<std::cell::RefCell<Box<dyn FnMut()>>>;

struct ScheduledSignal {
    id: u64,
    target: hecs::Entity,
    start: f32,
    end: std::cell::Cell<f32>,
    callback: SignalCallback,
}

struct SignalOverride {
    entity: hecs::Entity,
    type_id: std::any::TypeId,
    track_info: &'static TrackInfo,
    value: TrackValue,
}

#[derive(Default)]
pub(crate) struct SignalContext {
    next_id: std::cell::Cell<u64>,
    signals: std::cell::RefCell<Vec<ScheduledSignal>>,
    overrides: std::cell::RefCell<Vec<SignalOverride>>,
    evaluating: std::cell::Cell<bool>,
}

impl SignalContext {
    pub(crate) fn add(
        self: &std::rc::Rc<Self>,
        target: hecs::Entity,
        start: f32,
        callback: impl FnMut() + 'static,
        animator: AnimatorHandle,
    ) -> SignalHandle {
        let id = self.next_id.get();
        self.next_id
            .set(id.checked_add(1).expect("Too many signals."));
        self.signals.borrow_mut().push(ScheduledSignal {
            id,
            target,
            start,
            end: std::cell::Cell::new(f32::INFINITY),
            callback: std::rc::Rc::new(std::cell::RefCell::new(Box::new(callback))),
        });
        SignalHandle {
            id,
            context: std::rc::Rc::clone(self),
            animator,
        }
    }

    pub(crate) fn stop(&self, id: u64, end: f32) {
        let signals = self.signals.borrow();
        let signal = signals
            .iter()
            .find(|signal| signal.id == id)
            .expect("Signal handle must refer to an existing signal.");
        signal.end.set(signal.end.get().min(end));
    }

    pub(crate) fn is_evaluating(&self) -> bool {
        self.evaluating.get()
    }

    pub(crate) fn record_override(
        &self,
        entity: hecs::Entity,
        type_id: std::any::TypeId,
        track_info: &'static TrackInfo,
        value: TrackValue,
    ) {
        if !self.evaluating.get() {
            return;
        }

        let mut overrides = self.overrides.borrow_mut();
        if overrides.iter().any(|saved| {
            saved.entity == entity
                && saved.type_id == type_id
                && saved.track_info.id == track_info.id
        }) {
            return;
        }
        overrides.push(SignalOverride {
            entity,
            type_id,
            track_info,
            value,
        });
    }

    pub(crate) fn restore_overrides(&self, world: &hecs::World) {
        for saved in self.overrides.take() {
            if world.contains(saved.entity) {
                (saved.track_info.set)(world, saved.entity, saved.value);
            }
        }
    }

    pub(crate) fn evaluate(&self, world: &SceneWorld, time: f32) {
        let callbacks = {
            let world = world.borrow();
            self.signals
                .borrow()
                .iter()
                .filter(|signal| {
                    signal.start <= time
                        && time < signal.end.get()
                        && world
                            .get::<&Node>(signal.target)
                            .is_ok_and(|node| node.is_activated)
                })
                .map(|signal| std::rc::Rc::clone(&signal.callback))
                .collect::<Vec<_>>()
        };

        for callback in callbacks {
            assert!(
                !self.evaluating.replace(true),
                "Signals cannot be evaluated recursively."
            );
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                callback.borrow_mut()();
            }));
            self.evaluating.set(false);
            if let Err(error) = result {
                std::panic::resume_unwind(error);
            }
        }
    }
}

/// Controls the lifetime of a signal registered on a scene object.
pub struct SignalHandle {
    id: u64,
    context: std::rc::Rc<SignalContext>,
    animator: AnimatorHandle,
}

impl SignalHandle {
    /// Ends this signal at the current animator time.
    pub fn stop(&self) {
        self.animator.assert_timeline_mutation();
        self.context.stop(self.id, self.animator.time());
    }
}
