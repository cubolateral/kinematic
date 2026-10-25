use crate::core::types::{Color, Vector2};

pub(super) struct MorphPart {
    pub key: String,
    pub path: skia_safe::Path,
    pub color: Option<Color>,
}

pub(super) struct MorphDraw<'a> {
    pub part: &'a MorphPart,
    pub index: usize,
    pub incoming: bool,
    pub offset: Vector2,
    pub scale: Vector2,
    pub opacity: f32,
    pub color: Option<Color>,
    pub target_color: Option<Color>,
    pub progress: f32,
}

impl MorphDraw<'_> {
    pub fn color(&self, inherited: Color) -> Color {
        let from = self.color.unwrap_or(inherited).rgba();
        let to = self.target_color.unwrap_or(inherited).rgba();
        Color::from(std::array::from_fn::<_, 4, _>(|i| {
            from[i] + (to[i] - from[i]) * self.progress
        }))
    }

    pub fn path(&self) -> skia_safe::Path {
        self.part
            .path
            .with_transform(&skia_safe::Matrix::scale((self.scale.x, self.scale.y)))
            .with_offset((self.offset.x, self.offset.y))
    }

    pub fn transform(&self) -> glam::Mat4 {
        glam::Mat4::from_translation(self.offset.extend(0.0))
            * glam::Mat4::from_scale(self.scale.extend(1.0))
    }
}

pub(super) fn morph_parts<'a>(
    from: &'a [MorphPart],
    to: &'a [MorphPart],
    progress: f32,
) -> Vec<MorphDraw<'a>> {
    let progress = progress.clamp(0.0, 1.0);
    let mut used = vec![false; to.len()];
    let mut result = Vec::new();
    for (index, part) in from.iter().enumerate() {
        let mut draw = MorphDraw {
            part,
            index,
            incoming: false,
            offset: Vector2::ZERO,
            scale: Vector2::ONE,
            opacity: 1.0 - progress,
            color: part.color,
            target_color: part.color,
            progress,
        };
        if let Some((target_index, target)) = to
            .iter()
            .enumerate()
            .find(|(index, target)| !used[*index] && part.key == target.key)
        {
            used[target_index] = true;
            let a = part.path.compute_tight_bounds();
            let b = target.path.compute_tight_bounds();
            let ratio = |a: f32, b: f32| if a.abs() > f32::EPSILON { b / a } else { 1.0 };
            draw.scale = Vector2::ONE.lerp(
                Vector2::new(ratio(a.width(), b.width()), ratio(a.height(), b.height())),
                progress,
            );
            draw.offset = Vector2::new(a.center_x(), a.center_y())
                .lerp(Vector2::new(b.center_x(), b.center_y()), progress)
                - Vector2::new(a.center_x(), a.center_y()) * draw.scale;
            draw.opacity = 1.0;
            draw.target_color = target.color;
        }
        result.push(draw);
    }
    for (index, part) in to.iter().enumerate().filter(|(index, _)| !used[*index]) {
        result.push(MorphDraw {
            part,
            index,
            incoming: true,
            offset: Vector2::ZERO,
            scale: Vector2::ONE,
            opacity: progress,
            color: part.color,
            target_color: part.color,
            progress,
        });
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn part(key: &str, x: f32) -> MorphPart {
        MorphPart {
            key: key.into(),
            path: skia_safe::Path::rect(skia_safe::Rect::from_xywh(x, 0.0, 2.0, 4.0), None),
            color: None,
        }
    }

    #[test]
    fn repeated_glyphs_match_once_and_unmatched_glyphs_fade() {
        let from = [part("a", 0.0), part("a", 4.0), part("b", 8.0)];
        let to = [part("a", 8.0), part("c", 12.0)];
        let draws = morph_parts(&from, &to, 0.25);
        assert_eq!(draws.len(), 4);
        assert_eq!(draws[0].path().bounds().left, 2.0);
        assert_eq!(draws[0].opacity, 1.0);
        assert_eq!(draws[1].opacity, 0.75);
        assert_eq!(draws[2].opacity, 0.75);
        assert_eq!(draws[3].opacity, 0.25);
    }
}
