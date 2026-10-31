use dear_imgui_cte::{Palette, PaletteColor};
use serde::{Deserialize, Serialize};

macro_rules! themes {
    ($($variant:ident => ($name:literal, [$($color:expr),+]),)*) => {
        #[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
        pub(in crate::ui) enum Theme {
            #[default]
            Custom,
            $($variant,)*
        }

        impl Theme {
            pub const ALL: &[Self] = &[Self::Custom, $(Self::$variant,)*];
            pub const NAMES: &[&str] = &["Custom", $($name,)*];

            fn colors(self) -> Option<[u32; 12]> {
                match self {
                    Self::Custom => None,
                    $(Self::$variant => Some([$($color),+]),)*
                }
            }
        }
    };
}

// Background, text, comment, keyword, string, number, declaration, builtin, punctuation, selection, accent, error.
themes! {
    Dracula => ("Dracula", [0x282a36, 0xf8f8f2, 0x6272a4, 0xff79c6, 0xf1fa8c, 0xbd93f9, 0x50fa7b, 0x8be9fd, 0xf8f8f2, 0x44475a, 0xbd93f9, 0xff5555]),
    OneDark => ("One Dark", [0x282c34, 0xabb2bf, 0x5c6370, 0xc678dd, 0x98c379, 0xd19a66, 0x61afef, 0xe5c07b, 0xabb2bf, 0x3e4451, 0x61afef, 0xe06c75]),
    Monokai => ("Monokai", [0x272822, 0xf8f8f2, 0x75715e, 0xf92672, 0xe6db74, 0xae81ff, 0xa6e22e, 0x66d9ef, 0xf8f8f2, 0x49483e, 0xa6e22e, 0xf92672]),
    Nord => ("Nord", [0x2e3440, 0xd8dee9, 0x616e88, 0x81a1c1, 0xa3be8c, 0xb48ead, 0x88c0d0, 0x8fbcbb, 0xeceff4, 0x434c5e, 0x88c0d0, 0xbf616a]),
    SolarizedDark => ("Solarized Dark", [0x002b36, 0x839496, 0x586e75, 0x859900, 0x2aa198, 0xd33682, 0x268bd2, 0xb58900, 0x93a1a1, 0x073642, 0x268bd2, 0xdc322f]),
    SolarizedLight => ("Solarized Light", [0xfdf6e3, 0x657b83, 0x93a1a1, 0x859900, 0x2aa198, 0xd33682, 0x268bd2, 0xb58900, 0x586e75, 0xeee8d5, 0x268bd2, 0xdc322f]),
    GitHubDark => ("GitHub Dark", [0x0d1117, 0xe6edf3, 0x8b949e, 0xff7b72, 0xa5d6ff, 0x79c0ff, 0xd2a8ff, 0xffa657, 0xe6edf3, 0x264f78, 0x58a6ff, 0xf85149]),
    GitHubLight => ("GitHub Light", [0xffffff, 0x24292f, 0x6e7781, 0xcf222e, 0x0a3069, 0x0550ae, 0x8250df, 0x953800, 0x24292f, 0xb6d7ff, 0x0969da, 0xcf222e]),
    TokyoNight => ("Tokyo Night", [0x1a1b26, 0xc0caf5, 0x565f89, 0xbb9af7, 0x9ece6a, 0xff9e64, 0x7aa2f7, 0x7dcfff, 0x89ddff, 0x33467c, 0x7aa2f7, 0xf7768e]),
    CatppuccinMocha => ("Catppuccin Mocha", [0x1e1e2e, 0xcdd6f4, 0x6c7086, 0xcba6f7, 0xa6e3a1, 0xfab387, 0x89b4fa, 0xf9e2af, 0x9399b2, 0x45475a, 0x89b4fa, 0xf38ba8]),
    CatppuccinLatte => ("Catppuccin Latte", [0xeff1f5, 0x4c4f69, 0x9ca0b0, 0x8839ef, 0x40a02b, 0xfe640b, 0x1e66f5, 0xdf8e1d, 0x7c7f93, 0xccd0da, 0x1e66f5, 0xd20f39]),
    GruvboxDark => ("Gruvbox Dark", [0x282828, 0xebdbb2, 0x928374, 0xfb4934, 0xb8bb26, 0xd3869b, 0xfabd2f, 0x8ec07c, 0xebdbb2, 0x504945, 0x83a598, 0xfb4934]),
    GruvboxLight => ("Gruvbox Light", [0xfbf1c7, 0x3c3836, 0x928374, 0x9d0006, 0x79740e, 0x8f3f71, 0xb57614, 0x427b58, 0x3c3836, 0xd5c4a1, 0x076678, 0x9d0006]),
    RosePine => ("Rosé Pine", [0x191724, 0xe0def4, 0x6e6a86, 0x31748f, 0xf6c177, 0xebbcba, 0xebbcba, 0x9ccfd8, 0x908caa, 0x403d52, 0xc4a7e7, 0xeb6f92]),
    RosePineDawn => ("Rosé Pine Dawn", [0xfaf4ed, 0x575279, 0x9893a5, 0x286983, 0xea9d34, 0xd7827e, 0xd7827e, 0x56949f, 0x797593, 0xdfdad9, 0x907aa9, 0xb4637a]),
    EverforestDark => ("Everforest Dark", [0x2d353b, 0xd3c6aa, 0x859289, 0xe67e80, 0xa7c080, 0xd699b6, 0xa7c080, 0xdbbc7f, 0xd3c6aa, 0x543a48, 0x7fbbb3, 0xe67e80]),
    AyuDark => ("Ayu Dark", [0x0b0e14, 0xbfbdb6, 0x636a72, 0xff8f40, 0xaad94c, 0xd2a6ff, 0xffb454, 0x59c2ff, 0xbfbdb6, 0x253340, 0xe6b450, 0xf07178]),
    AyuLight => ("Ayu Light", [0xf8f9fa, 0x5c6166, 0x787b80, 0xfa8d3e, 0x86b300, 0xa37acc, 0xf2ae49, 0x399ee6, 0x5c6166, 0xd1e4f4, 0xff9940, 0xf07171]),
    MaterialPalenight => ("Material Palenight", [0x292d3e, 0xa6accd, 0x676e95, 0xc792ea, 0xc3e88d, 0xf78c6c, 0x82aaff, 0xffcb6b, 0x89ddff, 0x444267, 0x82aaff, 0xff5370]),
    NightOwl => ("Night Owl", [0x011627, 0xd6deeb, 0x637777, 0xc792ea, 0xecc48d, 0xf78c6c, 0x82aaff, 0xaddb67, 0xc5e4fd, 0x1d3b53, 0x7fdbca, 0xef5350]),
    SynthWave84 => ("SynthWave '84", [0x262335, 0xffffff, 0x848bbd, 0xfede5d, 0xff8b39, 0xf97e72, 0x36f9f6, 0xfe4450, 0xffffff, 0x463465, 0xff7edb, 0xfe4450]),
    Cobalt2 => ("Cobalt2", [0x193549, 0xffffff, 0x0088ff, 0xff9d00, 0xa5ff90, 0xff628c, 0xffc600, 0x80ffbb, 0xffffff, 0x0050a4, 0xffc600, 0xff628c]),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub(in crate::ui) struct Colors {
    pub theme: Theme,
    pub values: [[f32; 4]; PaletteColor::COUNT],
}

impl Default for Colors {
    fn default() -> Self {
        let mut palette = kinematic_ui::code::palette();
        palette.set(PaletteColor::Background, packed(0x1e1e1e));
        Self::from_palette(Theme::Custom, &palette)
    }
}

fn packed(rgb: u32) -> u32 {
    0xff000000 | ((rgb & 0xff) << 16) | (rgb & 0xff00) | ((rgb >> 16) & 0xff)
}

impl Colors {
    fn from_palette(theme: Theme, palette: &Palette) -> Self {
        Self {
            theme,
            values: palette.colors().map(|color| {
                std::array::from_fn(|channel| ((color >> (channel * 8)) & 0xff) as f32 / 255.0)
            }),
        }
    }

    pub fn apply(&mut self, theme: Theme) {
        let Some(
            [
                background,
                text,
                comment,
                keyword,
                string,
                number,
                declaration,
                builtin,
                punctuation,
                selection,
                accent,
                error,
            ],
        ) = theme.colors()
        else {
            self.theme = theme;
            return;
        };
        let mut palette = Palette::dark();
        for (slot, rgb) in [
            (PaletteColor::Text, text),
            (PaletteColor::Keyword, keyword),
            (PaletteColor::Declaration, declaration),
            (PaletteColor::Number, number),
            (PaletteColor::String, string),
            (PaletteColor::Punctuation, punctuation),
            (PaletteColor::Preprocessor, keyword),
            (PaletteColor::Identifier, text),
            (PaletteColor::KnownIdentifier, builtin),
            (PaletteColor::Comment, comment),
            (PaletteColor::Background, background),
            (PaletteColor::Cursor, text),
            (PaletteColor::Selection, selection),
            (PaletteColor::Whitespace, comment),
            (PaletteColor::MatchingBracketBackground, selection),
            (PaletteColor::MatchingBracketActive, accent),
            (PaletteColor::MatchingBracketLevel1, accent),
            (PaletteColor::MatchingBracketLevel2, string),
            (PaletteColor::MatchingBracketLevel3, number),
            (PaletteColor::MatchingBracketError, error),
            (PaletteColor::LineNumber, comment),
            (PaletteColor::CurrentLineNumber, text),
        ] {
            palette.set(slot, packed(rgb));
        }
        *self = Self::from_palette(theme, &palette);
    }

    pub fn draw(&mut self, ui: &dear_imgui_rs::Ui) -> bool {
        use kinematic_ui::{controls, properties};

        properties::section(ui, "Code Editor Colors");
        let mut theme = self.theme;
        let mut changed = properties::field(ui, "Theme", &mut theme, Theme::Custom, |value| {
            let mut index = Theme::ALL
                .iter()
                .position(|theme| theme == value)
                .unwrap_or(0);
            let changed = controls::choice(ui, "##value", &mut index, Theme::NAMES);
            *value = Theme::ALL[index];
            changed
        });
        if changed {
            self.apply(theme);
        }
        let mut defaults = Self::default();
        defaults.apply(self.theme);
        for slot in PaletteColor::ALL {
            let label = match slot {
                PaletteColor::Text => "Text",
                PaletteColor::Keyword => "Keywords",
                PaletteColor::Declaration => "Declarations",
                PaletteColor::Number => "Numbers",
                PaletteColor::String => "Strings",
                PaletteColor::Punctuation => "Punctuation",
                PaletteColor::Preprocessor => "Preprocessor",
                PaletteColor::Identifier => "Identifiers",
                PaletteColor::KnownIdentifier => "Known identifiers",
                PaletteColor::Comment => "Comments",
                PaletteColor::Background => "Code background",
                PaletteColor::Cursor => "Cursor",
                PaletteColor::Selection => "Selection",
                PaletteColor::Whitespace => "Whitespace and rulers",
                PaletteColor::MatchingBracketBackground => "Bracket background",
                PaletteColor::MatchingBracketActive => "Active bracket",
                PaletteColor::MatchingBracketLevel1 => "Bracket level 1",
                PaletteColor::MatchingBracketLevel2 => "Bracket level 2",
                PaletteColor::MatchingBracketLevel3 => "Bracket level 3",
                PaletteColor::MatchingBracketError => "Bracket error",
                PaletteColor::LineNumber => "Line numbers",
                PaletteColor::CurrentLineNumber => "Current line number",
            };
            let mut value = self.values[slot as usize];
            if properties::field(
                ui,
                label,
                &mut value,
                defaults.values[slot as usize],
                |value| controls::color(ui, "##value", value),
            ) {
                self.set(slot, value);
                changed = true;
            }
        }
        changed
    }

    pub fn set(&mut self, slot: PaletteColor, value: [f32; 4]) {
        self.values[slot as usize] = value;
        self.theme = Theme::Custom;
    }

    pub fn palette(&self) -> Palette {
        Palette::from_colors(self.values.map(|color| {
            color
                .iter()
                .enumerate()
                .fold(0, |packed, (channel, value)| {
                    packed | ((value.clamp(0.0, 1.0) * 255.0).round() as u32) << (channel * 8)
                })
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_customization_and_persistence() {
        let mut colors = Colors::default();
        for &theme in &Theme::ALL[1..] {
            colors.apply(theme);
            assert_eq!(colors.theme, theme);
            let palette = colors.palette();
            assert_ne!(
                palette.get(PaletteColor::Background),
                palette.get(PaletteColor::Text)
            );
            assert!(palette.colors().iter().all(|color| color >> 24 == 255));
        }
        colors.apply(Theme::Dracula);
        assert_eq!(colors.palette().get(PaletteColor::Background), 0xff362a28);
        let untouched = colors.values[PaletteColor::Keyword as usize];
        colors.set(PaletteColor::Background, [0.1, 0.2, 0.3, 1.0]);
        assert_eq!(colors.theme, Theme::Custom);
        assert_eq!(colors.values[PaletteColor::Keyword as usize], untouched);
        let saved = ron::to_string(&colors).unwrap();
        assert_eq!(ron::from_str::<Colors>(&saved).unwrap(), colors);
        colors.apply(Theme::OneDark);
        assert_eq!(colors.theme, Theme::OneDark);
        assert_eq!(colors.palette().get(PaletteColor::Background), 0xff342c28);
    }
}
