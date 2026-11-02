use sdl3::video::{Window, WindowFlags};
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    Graphics::Gdi::{GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow},
    UI::{
        Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
        WindowsAndMessaging::{MINMAXINFO, WM_GETMINMAXINFO, WM_NCDESTROY},
    },
};

pub(super) fn install(window: &Window) -> Result<(), String> {
    let hwnd = window_handle(window)?;

    // The callback stays attached only while SDL's native window is alive.
    if unsafe { SetWindowSubclass(hwnd, Some(window_proc), 0, window.raw() as usize) } == 0 {
        return Err(std::io::Error::last_os_error().to_string());
    }

    Ok(())
}

fn window_handle(window: &Window) -> Result<HWND, String> {
    let hwnd = unsafe {
        sdl3::sys::properties::SDL_GetPointerProperty(
            sdl3::sys::video::SDL_GetWindowProperties(window.raw()),
            sdl3::sys::video::SDL_PROP_WINDOW_WIN32_HWND_POINTER,
            std::ptr::null_mut(),
        )
    };

    if hwnd.is_null() {
        Err("Native window handle is unavailable.".into())
    } else {
        Ok(hwnd)
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    id: usize,
    window: usize,
) -> LRESULT {
    // Windows supplies the message data; SDL owns the window until WM_NCDESTROY.
    unsafe {
        if message == WM_NCDESTROY {
            RemoveWindowSubclass(hwnd, Some(window_proc), id);
        }

        let result = DefSubclassProc(hwnd, message, wparam, lparam);

        if message == WM_GETMINMAXINFO && lparam != 0 {
            let flags = WindowFlags::from(sdl3::sys::video::SDL_GetWindowFlags(
                window as *mut sdl3::sys::video::SDL_Window,
            ));

            if flags.contains(WindowFlags::BORDERLESS) && !flags.contains(WindowFlags::FULLSCREEN) {
                if let Some(monitor) = monitor_info(hwnd) {
                    set_maximized_bounds(&mut *(lparam as *mut MINMAXINFO), &monitor);

                    return 0;
                }
            }
        }

        result
    }
}

fn monitor_info(hwnd: HWND) -> Option<MONITORINFO> {
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..MONITORINFO::default()
    };

    unsafe {
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);

        (GetMonitorInfoW(monitor, &mut info) != 0).then_some(info)
    }
}

fn set_maximized_bounds(bounds: &mut MINMAXINFO, monitor: &MONITORINFO) {
    let work = monitor.rcWork;

    bounds.ptMaxPosition.x = work.left - monitor.rcMonitor.left;
    bounds.ptMaxPosition.y = work.top - monitor.rcMonitor.top;
    bounds.ptMaxSize.x = work.right - work.left;
    bounds.ptMaxSize.y = work.bottom - work.top;
}

#[cfg(test)]
mod tests {
    use super::{monitor_info, set_maximized_bounds, window_handle};
    use windows_sys::Win32::{
        Foundation::RECT,
        Graphics::Gdi::MONITORINFO,
        UI::WindowsAndMessaging::{MINMAXINFO, SendMessageW, WM_GETMINMAXINFO},
    };

    #[test]
    fn maximized_bounds_reserve_taskbars_on_offset_monitors() {
        let monitor = MONITORINFO {
            rcMonitor: RECT {
                left: -1920,
                top: -1080,
                right: 0,
                bottom: 0,
            },
            rcWork: RECT {
                left: -1880,
                top: -1050,
                right: 0,
                bottom: -40,
            },
            ..MONITORINFO::default()
        };
        let mut bounds = MINMAXINFO::default();

        set_maximized_bounds(&mut bounds, &monitor);

        assert_eq!((bounds.ptMaxPosition.x, bounds.ptMaxPosition.y), (40, 30));
        assert_eq!((bounds.ptMaxSize.x, bounds.ptMaxSize.y), (1880, 1010));
    }

    #[test]
    #[ignore = "Requires Windows desktop and SDL video."]
    fn borderless_window_maximizes_within_the_monitor_work_area() {
        let sdl = sdl3::init().unwrap();
        let video = sdl.video().unwrap();
        let mut window = video
            .window("Window bounds test", 640, 480)
            .hidden()
            .resizable()
            .build()
            .unwrap();
        let chrome = crate::app::window::Chrome::new(&mut window);
        let hwnd = window_handle(&window).unwrap();
        let monitor = monitor_info(hwnd).unwrap();
        let mut bounds = MINMAXINFO::default();

        assert!(chrome.controls(&window).visible);

        unsafe {
            SendMessageW(
                hwnd,
                WM_GETMINMAXINFO,
                0,
                (&mut bounds as *mut MINMAXINFO) as isize,
            );
        }

        assert_eq!(
            bounds.ptMaxSize.x,
            monitor.rcWork.right - monitor.rcWork.left
        );
        assert_eq!(
            bounds.ptMaxSize.y,
            monitor.rcWork.bottom - monitor.rcWork.top
        );
        assert_eq!(
            bounds.ptMaxPosition.x,
            monitor.rcWork.left - monitor.rcMonitor.left
        );
        assert_eq!(
            bounds.ptMaxPosition.y,
            monitor.rcWork.top - monitor.rcMonitor.top
        );
    }
}
