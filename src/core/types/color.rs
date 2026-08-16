/// RGBA color with channels stored in the sRGB color space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    /// Red channel from `0.0` to `1.0`.
    pub r: f32,
    /// Green channel from `0.0` to `1.0`.
    pub g: f32,
    /// Blue channel from `0.0` to `1.0`.
    pub b: f32,
    /// Alpha channel from `0.0` to `1.0`.
    pub a: f32,
}

impl Color {
    pub const TRANSPARENT: Self = Self::new(0.0, 0.0, 0.0, 0.0);
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0, 1.0);
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);
    pub const RED: Self = Self::new(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Self = Self::new(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Self = Self::new(0.0, 0.0, 1.0, 1.0);
    pub const YELLOW: Self = Self::new(1.0, 1.0, 0.0, 1.0);
    pub const CYAN: Self = Self::new(0.0, 1.0, 1.0, 1.0);
    pub const MAGENTA: Self = Self::new(1.0, 0.0, 1.0, 1.0);

    /// Creates an sRGB color from red, green, blue, and alpha channels.
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Returns the RGBA channels in red, green, blue, alpha order.
    pub fn rgba(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

impl From<[f32; 4]> for Color {
    fn from([r, g, b, a]: [f32; 4]) -> Self {
        Self::new(r, g, b, a)
    }
}

impl From<palette::Srgba> for Color {
    fn from(color: palette::Srgba) -> Self {
        Self::new(
            color.color.red,
            color.color.green,
            color.color.blue,
            color.alpha,
        )
    }
}

impl From<Color> for palette::Srgba {
    fn from(color: Color) -> Self {
        Self::new(color.r, color.g, color.b, color.a)
    }
}
