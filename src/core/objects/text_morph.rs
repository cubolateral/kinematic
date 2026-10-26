use crate::core::types::{Color, Vector2};

pub(super) struct MorphPart {
    pub key: String,
    pub group: usize,
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

#[derive(Default)]
pub(super) struct TokenGroups {
    index: usize,
    word: bool,
}

impl TokenGroups {
    pub fn next(&mut self, ch: char) -> usize {
        let word = ch.is_alphanumeric() || ch == '_';
        if !word || !self.word {
            self.index += 1;
        }
        self.word = word;
        self.index
    }
}

pub(super) fn morph_parts<'a>(
    from: &'a [MorphPart],
    to: &'a [MorphPart],
    progress: f32,
    group: bool,
) -> Vec<MorphDraw<'a>> {
    let ranges = |parts: &[MorphPart]| {
        let mut ranges: Vec<std::ops::Range<usize>> = Vec::new();
        for (index, part) in parts.iter().enumerate() {
            if group && index > 0 && part.group == parts[index - 1].group {
                ranges.last_mut().unwrap().end = index + 1;
            } else {
                ranges.push(index..index + 1);
            }
        }
        ranges
    };
    let bounds = |parts: &[MorphPart]| {
        let mut bounds = parts[0].path.compute_tight_bounds();
        for part in &parts[1..] {
            bounds.join(part.path.compute_tight_bounds());
        }
        bounds
    };
    let progress = progress.clamp(0.0, 1.0);
    let targets = ranges(to);
    let mut used = vec![false; targets.len()];
    let mut result = Vec::new();
    for source in ranges(from) {
        let parts = &from[source.clone()];
        let target = targets.iter().enumerate().find(|(index, range)| {
            !used[*index]
                && parts
                    .iter()
                    .map(|p| &p.key)
                    .eq(to[(*range).clone()].iter().map(|p| &p.key))
        });
        let mut scale = Vector2::ONE;
        let mut offset = Vector2::ZERO;
        if let Some((index, range)) = target {
            used[index] = true;
            let a = bounds(parts);
            let b = bounds(&to[range.clone()]);
            let ratio = |a: f32, b: f32| if a.abs() > f32::EPSILON { b / a } else { 1.0 };
            scale = Vector2::ONE.lerp(
                Vector2::new(ratio(a.width(), b.width()), ratio(a.height(), b.height())),
                progress,
            );
            offset = Vector2::new(a.center_x(), a.center_y())
                .lerp(Vector2::new(b.center_x(), b.center_y()), progress)
                - Vector2::new(a.center_x(), a.center_y()) * scale;
        }
        for (index, part) in parts.iter().enumerate() {
            result.push(MorphDraw {
                part,
                index: source.start + index,
                incoming: false,
                offset,
                scale,
                opacity: if target.is_some() {
                    1.0
                } else {
                    1.0 - progress
                },
                color: part.color,
                target_color: target.map_or(part.color, |(_, range)| to[range.start + index].color),
                progress,
            });
        }
    }
    for (_, range) in targets
        .iter()
        .enumerate()
        .filter(|(index, _)| !used[*index])
    {
        for index in range.clone() {
            let part = &to[index];
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
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn part(key: &str, x: f32) -> MorphPart {
        MorphPart {
            key: key.into(),
            group: 0,
            path: skia_safe::Path::rect(skia_safe::Rect::from_xywh(x, 0.0, 2.0, 4.0), None),
            color: None,
        }
    }

    fn words(text: &str) -> Vec<MorphPart> {
        let mut groups = TokenGroups::default();
        text.chars()
            .enumerate()
            .filter_map(|(i, ch)| {
                let group = groups.next(ch);
                (!ch.is_whitespace()).then(|| MorphPart {
                    group,
                    ..part(&ch.to_string(), i as f32 * 4.0)
                })
            })
            .collect()
    }

    #[test]
    fn groups_match_whole_words_and_separate_symbols() {
        let from = words("local d = wji make:func é_2");
        let mut tokens: Vec<String> = Vec::new();
        for (i, part) in from.iter().enumerate() {
            if i == 0 || part.group != from[i - 1].group {
                tokens.push(String::new());
            }
            tokens.last_mut().unwrap().push_str(&part.key);
        }
        assert_eq!(
            tokens,
            ["local", "d", "=", "wji", "make", ":", "func", "é_2"]
        );

        let from = words("make:func");
        let to = words("func:maker");
        let draws = morph_parts(&from, &to, 0.5, true);
        assert!(draws[..4].iter().all(|draw| draw.opacity == 0.5));
        assert!(draws[4..9].iter().all(|draw| draw.opacity == 1.0));
        assert!(
            draws[5..9]
                .iter()
                .all(|draw| draw.offset == draws[5].offset)
        );
        assert!(
            morph_parts(&from, &to, 0.5, false)[..9]
                .iter()
                .all(|draw| draw.opacity == 1.0)
        );

        let from = words("aa aa");
        let to = words("aa");
        let draws = morph_parts(&from, &to, 0.5, true);
        assert!(draws[..2].iter().all(|draw| draw.opacity == 1.0));
        assert!(draws[2..].iter().all(|draw| draw.opacity == 0.5));
    }

    #[test]
    fn repeated_glyphs_match_once_and_unmatched_glyphs_fade() {
        let from = [part("a", 0.0), part("a", 4.0), part("b", 8.0)];
        let to = [part("a", 8.0), part("c", 12.0)];
        let draws = morph_parts(&from, &to, 0.25, false);
        assert_eq!(draws.len(), 4);
        assert_eq!(draws[0].path().bounds().left, 2.0);
        assert_eq!(draws[0].opacity, 1.0);
        assert_eq!(draws[1].opacity, 0.75);
        assert_eq!(draws[2].opacity, 0.75);
        assert_eq!(draws[3].opacity, 0.25);
    }
}
