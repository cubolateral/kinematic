use std::{
    any::Any,
    cell::RefCell,
    ffi::c_void,
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
};

use sdl3::sys::{events, init};

struct Watch<'a> {
    window_id: u32,
    redraw: RefCell<&'a mut dyn FnMut()>,
    panic: RefCell<Option<Box<dyn Any + Send>>>,
}

struct Registration {
    data: *mut c_void,
    registered: bool,
}

impl Drop for Registration {
    fn drop(&mut self) {
        if self.registered {
            // The stack callback must be removed before its captures are released.
            unsafe { events::SDL_RemoveEventWatch(Some(redraw), self.data) };
        }
    }
}

pub(super) fn poll<R>(window_id: u32, mut draw: impl FnMut(), pump: impl FnOnce() -> R) -> R {
    let watch = Watch {
        window_id,
        redraw: RefCell::new(&mut draw),
        panic: RefCell::new(None),
    };
    let data = (&watch as *const Watch<'_>).cast_mut().cast();
    let registration = Registration {
        data,
        registered: unsafe { events::SDL_AddEventWatch(Some(redraw), data) },
    };
    let result = pump();

    drop(registration);

    if let Some(panic) = watch.panic.into_inner() {
        resume_unwind(panic);
    }

    result
}

unsafe extern "C" fn redraw(data: *mut c_void, event: *mut events::SDL_Event) -> bool {
    // SDL can notify watchers from other threads; graphics stay on the main thread.
    unsafe {
        if !init::SDL_IsMainThread() || (*event).r#type != events::SDL_EVENT_WINDOW_EXPOSED.0 {
            return true;
        }

        let watch = &*(data as *const Watch<'_>);

        if (*event).window.windowID != watch.window_id || watch.panic.borrow().is_some() {
            return true;
        }

        if let Ok(mut draw) = watch.redraw.try_borrow_mut() {
            // Rendering can itself emit events; nested notifications must not enter ImGui again.
            if let Err(panic) = catch_unwind(AssertUnwindSafe(|| draw())) {
                *watch.panic.borrow_mut() = Some(panic);
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use sdl3::event::{Event, WindowEvent};

    #[test]
    #[ignore = "Requires SDL video on the main thread."]
    fn redraw_stays_on_the_main_thread_and_is_scoped_and_non_recursive() {
        let sdl = sdl3::init().unwrap();
        let video = sdl.video().unwrap();
        let window = video
            .window("Live redraw test", 640, 480)
            .hidden()
            .build()
            .unwrap();
        let events = sdl.event().unwrap();
        let calls = Cell::new(0);
        let exposed = Event::Window {
            timestamp: 0,
            window_id: window.id(),
            win_event: WindowEvent::Exposed,
        };

        super::poll(
            window.id(),
            || {
                calls.set(calls.get() + 1);

                events.push_event(exposed.clone()).unwrap();
            },
            || {
                let sender = events.event_sender();
                let background_event = exposed.clone();

                std::thread::spawn(move || sender.push_event(background_event).unwrap())
                    .join()
                    .unwrap();

                assert_eq!(calls.get(), 0);

                events.push_event(exposed.clone()).unwrap();

                assert_eq!(calls.get(), 1);
            },
        );

        events.push_event(exposed.clone()).unwrap();

        assert_eq!(calls.get(), 1);

        let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            super::poll(
                window.id(),
                || panic!("Redraw failed."),
                || events.push_event(exposed.clone()).unwrap(),
            );
        }));

        assert!(failure.is_err());

        events.push_event(exposed).unwrap();
    }
}
