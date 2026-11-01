use dear_imgui_rs::Ui;
use kinematic_ui::controls;

#[derive(Clone, Copy, Default, PartialEq)]
pub(super) enum Resolution {
    Sd,
    #[default]
    Hd,
    FullHd,
    Qhd,
    Uhd,
    Square,
    PortraitHd,
    PortraitFullHd,
}

impl Resolution {
    const OPTIONS: [Self; 8] = [
        Self::Sd,
        Self::Hd,
        Self::FullHd,
        Self::Qhd,
        Self::Uhd,
        Self::Square,
        Self::PortraitHd,
        Self::PortraitFullHd,
    ];

    pub(super) fn value(self) -> (u32, u32) {
        match self {
            Self::Sd => (854, 480),
            Self::Hd => (1280, 720),
            Self::FullHd => (1920, 1080),
            Self::Qhd => (2560, 1440),
            Self::Uhd => (3840, 2160),
            Self::Square => (1080, 1080),
            Self::PortraitHd => (720, 1280),
            Self::PortraitFullHd => (1080, 1920),
        }
    }

    pub(super) fn draw(&mut self, ui: &Ui) -> bool {
        let mut index = Self::OPTIONS
            .iter()
            .position(|value| value == self)
            .unwrap();
        let labels = Self::OPTIONS.map(|value| {
            let (width, height) = value.value();
            let description = match value {
                Self::Sd => "SD",
                Self::Hd => "HD",
                Self::FullHd => "Full HD",
                Self::Qhd => "QHD",
                Self::Uhd => "4K UHD",
                Self::Square => "Square",
                Self::PortraitHd => "HD portrait",
                Self::PortraitFullHd => "Full HD portrait",
            };
            format!("{width} × {height} ({description})")
        });
        let changed = controls::choice(ui, "##Resolution", &mut index, &labels);
        *self = Self::OPTIONS[index];
        changed
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
pub(super) enum Fps {
    Fps12 = 12,
    Fps24 = 24,
    Fps25 = 25,
    Fps30 = 30,
    Fps50 = 50,
    #[default]
    Fps60 = 60,
    Fps120 = 120,
}

impl Fps {
    const OPTIONS: [Self; 7] = [
        Self::Fps12,
        Self::Fps24,
        Self::Fps25,
        Self::Fps30,
        Self::Fps50,
        Self::Fps60,
        Self::Fps120,
    ];

    pub(super) fn value(self) -> u32 {
        self as u32
    }

    pub(super) fn draw(&mut self, ui: &Ui) -> bool {
        let mut index = Self::OPTIONS
            .iter()
            .position(|value| value == self)
            .unwrap();
        let labels = Self::OPTIONS.map(|value| value.value().to_string());
        let changed = controls::choice(ui, "##FPS", &mut index, &labels);
        *self = Self::OPTIONS[index];
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_have_the_project_defaults_and_valid_export_settings() {
        let defaults = crate::core::ProjectSettings::default();
        assert_eq!(Resolution::default().value(), defaults.resolution);
        assert_eq!(Fps::default().value(), defaults.fps);
        for resolution in Resolution::OPTIONS {
            for fps in Fps::OPTIONS {
                assert!(crate::core::ProjectSettings::is_valid(
                    &crate::core::ProjectSettings {
                        resolution: resolution.value(),
                        fps: fps.value(),
                        ..defaults
                    }
                ));
            }
        }
    }
}
