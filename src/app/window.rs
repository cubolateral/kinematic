use std::{cell::Cell, rc::Rc};

use kinematic_ui::title_bar::{Action, Controls};
use sdl3::video::{FullscreenType, HitTestResult, Window, WindowFlags};

#[cfg(target_os = "windows")]
mod windows;

pub(super) struct Chrome {
    area: Rc<Cell<HitArea>>,
    custom: bool,
    minimize: bool,
    maximize: bool,
}

#[derive(Clone, Copy, Default)]
struct HitArea {
    size: [f32; 2],
    border: f32,
    drag: Option<[[f32; 2]; 2]>,
}

impl Chrome {
    pub fn new(window: &mut Window) -> Self {
        let area = Rc::new(Cell::new(HitArea::default()));
        let callback_area = area.clone();
        let custom = match window.set_hit_test(move |point| {
            callback_area
                .get()
                .hit([point.x() as f32, point.y() as f32])
        }) {
            Ok(()) => window.set_bordered(false),
            Err(error) => {
                eprintln!("Custom window decorations unavailable: {error}.");

                false
            }
        };
        #[cfg(target_os = "windows")]
        let custom = if custom {
            match windows::install(window) {
                Ok(()) => true,
                Err(error) => {
                    eprintln!("Custom window decorations unavailable: {error}.");

                    window.set_bordered(true);

                    false
                }
            }
        } else {
            false
        };
        let (minimize, maximize) = if cfg!(target_os = "linux") {
            session_actions(&desktop_session())
        } else {
            (true, true)
        };

        // Some Wayland compositors use the minimum as the initial floating size.
        if custom && window.subsystem().current_video_driver() != "wayland" {
            if let Err(error) = window.set_minimum_size(320, 240) {
                eprintln!("Could not set the minimum window size: {error}.");
            }
        }

        Self {
            area,
            custom,
            minimize,
            maximize,
        }
    }

    pub fn controls(&self, window: &Window) -> Controls {
        let fullscreen = window.fullscreen_state() != FullscreenType::Off;

        Controls {
            visible: self.custom && !fullscreen,
            minimize: self.minimize,
            maximize: self.maximize
                && WindowFlags::from(window.window_flags()).contains(WindowFlags::RESIZABLE)
                && !fullscreen,
            maximized: window.is_maximized(),
            macos: cfg!(target_os = "macos"),
        }
    }

    pub fn update(&self, window: &Window, drag: Option<[[f32; 2]; 2]>, scale: f32) {
        let (width, height) = window.size();
        let controls = self.controls(window);

        self.area.set(HitArea {
            size: [width as f32, height as f32],
            border: if controls.visible
                && WindowFlags::from(window.window_flags()).contains(WindowFlags::RESIZABLE)
                && !controls.maximized
            {
                4.0 * scale
            } else {
                0.0
            },
            drag: if controls.visible { drag } else { None },
        });
    }

    pub fn apply(&self, window: &mut Window, action: Action) -> Result<(), String> {
        let succeeded = match action {
            Action::Minimize if self.minimize => window.minimize(),
            Action::ToggleMaximize if self.controls(window).maximize => {
                if window.is_maximized() {
                    window.restore()
                } else {
                    window.maximize()
                }
            }
            _ => return Ok(()),
        };

        if succeeded {
            Ok(())
        } else {
            Err(format!(
                "Could not change the window state: {}.",
                sdl3::get_error()
            ))
        }
    }
}

impl HitArea {
    fn hit(self, [x, y]: [f32; 2]) -> HitTestResult {
        let [width, height] = self.size;

        if x < 0.0 || y < 0.0 || x >= width || y >= height {
            return HitTestResult::Normal;
        }

        if self.border > 0.0 {
            let left = x < self.border;
            let right = x >= width - self.border;
            let top = y < self.border;
            let bottom = y >= height - self.border;

            match (left, right, top, bottom) {
                (true, _, true, _) => return HitTestResult::ResizeTopLeft,
                (_, true, true, _) => return HitTestResult::ResizeTopRight,
                (true, _, _, true) => return HitTestResult::ResizeBottomLeft,
                (_, true, _, true) => return HitTestResult::ResizeBottomRight,
                (true, _, _, _) => return HitTestResult::ResizeLeft,
                (_, true, _, _) => return HitTestResult::ResizeRight,
                (_, _, true, _) => return HitTestResult::ResizeTop,
                (_, _, _, true) => return HitTestResult::ResizeBottom,
                _ => {}
            }
        }

        if let Some([min, max]) = self.drag {
            if x >= min[0] && x < max[0] && y >= min[1] && y < max[1] {
                return HitTestResult::Draggable;
            }
        }

        HitTestResult::Normal
    }
}

fn desktop_session() -> String {
    [
        "XDG_CURRENT_DESKTOP",
        "XDG_SESSION_DESKTOP",
        "DESKTOP_SESSION",
    ]
    .into_iter()
    .find_map(|name| std::env::var(name).ok().filter(|value| !value.is_empty()))
    .or_else(|| {
        [
            ("HYPRLAND_INSTANCE_SIGNATURE", "Hyprland"),
            ("SWAYSOCK", "sway"),
            ("NIRI_SOCKET", "niri"),
        ]
        .into_iter()
        .find_map(|(variable, name)| std::env::var_os(variable).map(|_| name.to_owned()))
    })
    .unwrap_or_default()
}

fn session_actions(session: &str) -> (bool, bool) {
    // SDL does not expose Wayland's per-window management capabilities.
    let mut minimize = true;
    let mut maximize = true;

    for name in session.split(':') {
        match name.trim().to_ascii_lowercase().as_str() {
            "niri" => minimize = false,
            "hyprland" | "sway" | "river" | "i3" | "bspwm" | "dwm" => {
                minimize = false;
                maximize = false;
            }
            _ => {}
        }
    }

    (minimize, maximize)
}

#[cfg(test)]
mod tests {
    use super::{HitArea, session_actions};
    use sdl3::video::HitTestResult;

    #[test]
    fn title_bar_keeps_buttons_and_content_outside_the_drag_area() {
        let mut area = HitArea {
            size: [800.0, 600.0],
            border: 4.0,
            drag: Some([[200.0, 0.0], [650.0, 28.0]]),
        };

        assert_eq!(area.hit([300.0, 12.0]), HitTestResult::Draggable);
        assert_eq!(area.hit([40.0, 12.0]), HitTestResult::Normal);
        assert_eq!(area.hit([700.0, 12.0]), HitTestResult::Normal);
        assert_eq!(area.hit([300.0, 40.0]), HitTestResult::Normal);
        assert_eq!(area.hit([1.0, 1.0]), HitTestResult::ResizeTopLeft);
        assert_eq!(area.hit([798.0, 598.0]), HitTestResult::ResizeBottomRight);

        area.border = 0.0;
        area.drag = None;

        assert_eq!(area.hit([1.0, 1.0]), HitTestResult::Normal);
        assert_eq!(area.hit([300.0, 12.0]), HitTestResult::Normal);
    }

    #[test]
    fn window_actions_follow_the_desktop_session() {
        assert_eq!(session_actions("Hyprland"), (false, false));
        assert_eq!(session_actions("niri"), (false, true));
        assert_eq!(session_actions("sway"), (false, false));
        assert_eq!(session_actions("GNOME"), (true, true));
        assert_eq!(session_actions("KDE"), (true, true));
        assert_eq!(session_actions("custom-sway-session"), (true, true));
    }
}
