use super::{Vector2, vec2};

/// Rectangle in scene coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn middle(self) -> Vector2 {
        vec2(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
    pub fn top(self) -> Vector2 {
        vec2(self.x + self.width / 2.0, self.y)
    }
    pub fn bottom(self) -> Vector2 {
        vec2(self.x + self.width / 2.0, self.y + self.height)
    }
    pub fn left(self) -> Vector2 {
        vec2(self.x, self.y + self.height / 2.0)
    }
    pub fn right(self) -> Vector2 {
        vec2(self.x + self.width, self.y + self.height / 2.0)
    }
    pub fn top_left(self) -> Vector2 {
        vec2(self.x, self.y)
    }
    pub fn top_right(self) -> Vector2 {
        vec2(self.x + self.width, self.y)
    }
    pub fn bottom_left(self) -> Vector2 {
        vec2(self.x, self.y + self.height)
    }
    pub fn bottom_right(self) -> Vector2 {
        vec2(self.x + self.width, self.y + self.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchors_follow_rectangle_edges() {
        let rect = Rect::new(-40.0, -20.0, 80.0, 40.0);
        assert_eq!(rect.middle(), vec2(0.0, 0.0));
        assert_eq!(rect.top(), vec2(0.0, -20.0));
        assert_eq!(rect.bottom(), vec2(0.0, 20.0));
        assert_eq!(rect.left(), vec2(-40.0, 0.0));
        assert_eq!(rect.right(), vec2(40.0, 0.0));
        assert_eq!(rect.top_left(), vec2(-40.0, -20.0));
        assert_eq!(rect.bottom_right(), vec2(40.0, 20.0));
    }
}
