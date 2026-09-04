use crate::core::{Scene, Task, components::Animation};

#[doc(hidden)]
#[derive(Clone)]
pub struct AnimatorHandle {
    state: std::rc::Rc<std::cell::RefCell<AnimatorState>>,
    context: std::rc::Rc<AnimatorContext>,
}

struct AnimatorContext {
    time: std::rc::Rc<std::cell::Cell<f32>>,
    active: std::cell::RefCell<Option<AnimatorHandle>>,
}

struct AnimatorState {
    tasks: Vec<Task>,
    start_time: f32,
    elapsed: f32,
    scheduling: Scheduling,
}

pub(crate) struct Animator {
    handle: AnimatorHandle,
}

#[derive(Clone, Copy)]
pub(crate) enum Scheduling {
    Sequential,
    Parallel,
}

impl Animator {
    pub(crate) fn with_scene_time(scene_time: std::rc::Rc<std::cell::Cell<f32>>) -> Self {
        scene_time.set(0.0);

        let context = std::rc::Rc::new(AnimatorContext {
            time: scene_time,
            active: std::cell::RefCell::new(None),
        });

        let handle = AnimatorHandle {
            state: std::rc::Rc::new(std::cell::RefCell::new(AnimatorState {
                tasks: vec![],
                start_time: 0.0,
                elapsed: 0.0,
                scheduling: Scheduling::Sequential,
            })),
            context: std::rc::Rc::clone(&context),
        };
        context.active.replace(Some(handle.clone()));

        Self { handle }
    }

    pub(crate) fn handle(&self) -> AnimatorHandle {
        self.handle.clone()
    }

    pub(crate) fn group(&self, scheduling: Scheduling) -> Self {
        let state = self.handle.state.borrow();
        let start_time = match state.scheduling {
            Scheduling::Sequential => self.get_time(),
            Scheduling::Parallel => state.start_time,
        };

        Self {
            handle: AnimatorHandle {
                state: std::rc::Rc::new(std::cell::RefCell::new(AnimatorState {
                    tasks: vec![],
                    start_time,
                    elapsed: 0.0,
                    scheduling,
                })),
                context: std::rc::Rc::clone(&self.handle.context),
            },
        }
    }

    fn get_time(&self) -> f32 {
        let state = self.handle.state.borrow();
        state.start_time + state.elapsed
    }

    pub(crate) fn get_duration_for_tasks(tasks: &[Task], scene: &mut Scene) -> f32 {
        let mut time = 0.0;

        for task in tasks {
            time += Self::process_task(scene, task, time);
        }

        time
    }

    /// Adds a task's keyframes at `start_time` and returns its elapsed duration.
    fn process_task(scene: &mut Scene, task: &Task, start_time: f32) -> f32 {
        match task {
            Task::Tween {
                entity,
                type_id,
                track_info,
                from,
                to,
                duration,
                easing,
            } => {
                scene
                    .get_world()
                    .get::<&mut Animation>(*entity)
                    .unwrap()
                    .animate(
                        start_time,
                        *type_id,
                        track_info,
                        from.clone(),
                        to.clone(),
                        *duration,
                        *easing,
                    );

                *duration
            }

            Task::Wait(duration) => *duration,

            Task::Chain(tasks) => {
                let mut duration = 0.0;

                for task in tasks {
                    duration += Self::process_task(scene, task, start_time + duration);
                }

                duration
            }

            Task::All(tasks) => {
                let mut max_duration: f32 = 0.0;

                for task in tasks {
                    let duration = Self::process_task(scene, task, start_time);

                    max_duration = max_duration.max(duration);
                }

                max_duration
            }

            Task::Repeat(repetitions, tasks) => {
                let mut duration = 0.0;

                for _ in 0..*repetitions {
                    for task in tasks {
                        duration += Self::process_task(scene, task, start_time + duration);
                    }
                }

                duration
            }
        }
    }

    pub(crate) fn tasks(&self) -> Vec<Task> {
        self.handle.state.borrow().tasks.clone()
    }

    pub(crate) fn task_duration(task: &Task) -> f32 {
        match task {
            Task::Tween { duration, .. } | Task::Wait(duration) => *duration,
            Task::Chain(tasks) => tasks.iter().map(Self::task_duration).sum(),
            Task::All(tasks) => tasks.iter().map(Self::task_duration).fold(0.0, f32::max),
            Task::Repeat(repetitions, tasks) => {
                *repetitions as f32 * tasks.iter().map(Self::task_duration).sum::<f32>()
            }
        }
    }
}

impl AnimatorHandle {
    pub(crate) fn activate(&self) -> Option<AnimatorHandle> {
        let previous = self.context.active.replace(Some(self.clone()));
        self.sync_scene_time();
        previous
    }

    pub(crate) fn restore(&self, previous: Option<AnimatorHandle>) {
        if let Some(previous) = previous {
            self.context.active.replace(Some(previous.clone()));
            previous.sync_scene_time();
        }
    }

    pub(crate) fn active(&self) -> Self {
        self.context
            .active
            .borrow()
            .as_ref()
            .cloned()
            .unwrap_or_else(|| self.clone())
    }

    pub(crate) fn time(&self) -> f32 {
        self.context.time.get()
    }

    fn sync_scene_time(&self) {
        let state = self.state.borrow();
        self.context.time.set(state.start_time + state.elapsed);
    }

    fn is_active(&self) -> bool {
        self.context
            .active
            .borrow()
            .as_ref()
            .is_some_and(|active| std::rc::Rc::ptr_eq(&active.state, &self.state))
    }

    pub(crate) fn play(&self, task: Task) {
        let duration = Animator::task_duration(&task);
        let mut state = self.state.borrow_mut();
        state.tasks.push(task);

        match state.scheduling {
            Scheduling::Sequential => state.elapsed += duration,
            Scheduling::Parallel => state.elapsed = state.elapsed.max(duration),
        }

        if self.is_active() {
            self.context.time.set(state.start_time + state.elapsed);
        }
    }
}
