use crate::core::types::{Vector2, vec2};

pub(crate) fn regular_polygon_vertices(sides: u32) -> Option<Vec<Vector2>> {
    if !(3..=256).contains(&sides) {
        return None;
    }

    let start_angle = -std::f32::consts::PI + std::f32::consts::PI / sides as f32;
    let angles: Vec<_> = (0..sides)
        .map(|side| start_angle + std::f32::consts::TAU * side as f32 / sides as f32)
        .collect();
    let maximum_coordinate = angles
        .iter()
        .map(|angle| angle.cos().abs().max(angle.sin().abs()))
        .fold(0.0, f32::max);
    let radius = 0.5 / maximum_coordinate;

    Some(
        angles
            .into_iter()
            .map(|angle| vec2(radius * angle.cos(), radius * angle.sin()))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regular_polygon_vertices_validate_sides_and_fill_the_default_bounds() {
        assert!(regular_polygon_vertices(2).is_none());
        assert_eq!(regular_polygon_vertices(3).unwrap().len(), 3);

        let square = regular_polygon_vertices(4).unwrap();
        for vertex in square {
            assert!((vertex.x.abs() - 0.5).abs() <= f32::EPSILON);
            assert!((vertex.y.abs() - 0.5).abs() <= f32::EPSILON);
        }
    }
}
