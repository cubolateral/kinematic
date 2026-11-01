/// Four corner values in top-left, top-right, bottom-right, bottom-left order.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Quad {
    /// Top-left value.
    pub a: f32,
    /// Top-right value.
    pub b: f32,
    /// Bottom-right value.
    pub c: f32,
    /// Bottom-left value.
    pub d: f32,
}

impl Quad {
    /// Creates four explicit corner values.
    pub const fn new(a: f32, b: f32, c: f32, d: f32) -> Self {
        Self { a, b, c, d }
    }

    /// Returns the values in corner order.
    pub const fn to_array(self) -> [f32; 4] {
        [self.a, self.b, self.c, self.d]
    }

    /// Interpolates each value independently.
    pub fn lerp(self, to: Self, t: f32) -> Self {
        Self::new(
            self.a + (to.a - self.a) * t,
            self.b + (to.b - self.b) * t,
            self.c + (to.c - self.c) * t,
            self.d + (to.d - self.d) * t,
        )
    }
}

impl From<f32> for Quad {
    fn from(value: f32) -> Self {
        Self::new(value, value, value, value)
    }
}

impl From<[f32; 2]> for Quad {
    fn from([a, b]: [f32; 2]) -> Self {
        Self::new(a, b, a, b)
    }
}

impl From<[f32; 3]> for Quad {
    fn from([a, b, c]: [f32; 3]) -> Self {
        Self::new(a, b, c, b)
    }
}

impl From<[f32; 4]> for Quad {
    fn from([a, b, c, d]: [f32; 4]) -> Self {
        Self::new(a, b, c, d)
    }
}

impl From<i32> for Quad {
    fn from(value: i32) -> Self {
        Self::from(value as f32)
    }
}

impl<const N: usize> From<[i32; N]> for Quad
where
    [f32; N]: Into<Quad>,
{
    fn from(values: [i32; N]) -> Self {
        values.map(|value| value as f32).into()
    }
}

#[cfg(test)]
mod tests {
    use super::Quad;

    #[test]
    fn shorthand_expands_in_corner_order() {
        assert_eq!(Quad::from(10.0), Quad::new(10.0, 10.0, 10.0, 10.0));
        assert_eq!(Quad::from([10.0, 30.0]), Quad::new(10.0, 30.0, 10.0, 30.0));
        assert_eq!(
            Quad::from([10.0, 20.0, 30.0]),
            Quad::new(10.0, 20.0, 30.0, 20.0)
        );
        assert_eq!(
            Quad::from([10.0, 20.0, 30.0, 40.0]),
            Quad::new(10.0, 20.0, 30.0, 40.0)
        );
    }
}
