//! Interactive viewport orientation axes.

use dear_imgui_rs::{MouseButton, Ui};

const SIZE: f32 = 104.0;
const ARM: f32 = 33.0;
const RADIUS: f32 = 8.0;
const HOVER_RADIUS: f32 = RADIUS + 2.0;
const OUTLINE: f32 = 1.5;
const COLORS: [[f32; 4]; 3] = [
    [0.95, 0.25, 0.32, 1.0],
    [0.55, 0.8, 0.15, 1.0],
    [0.2, 0.55, 1.0, 1.0],
];

#[derive(Default)]
pub struct State {
    pressed: Option<(usize, bool)>,
    started: bool,
    dragged: bool,
}

#[derive(Default)]
pub struct Response {
    pub axis: Option<(usize, bool)>,
    pub orbit: Option<[f32; 2]>,
}

pub fn bounds(min: [f32; 2], size: [f32; 2]) -> [[f32; 2]; 2] {
    let origin = [min[0] + size[0] - SIZE - 8.0, min[1] + 8.0];
    [origin, [origin[0] + SIZE, origin[1] + SIZE]]
}

fn points(axes: [[f32; 3]; 3], center: [f32; 2]) -> [(usize, bool, [f32; 3]); 6] {
    let mut points = std::array::from_fn(|index| {
        let axis = index / 2;
        let positive = index % 2 == 0;
        let sign = if positive { 1.0 } else { -1.0 };
        let [x, y, z] = axes[axis];
        (
            axis,
            positive,
            [
                center[0] + x * sign * ARM,
                center[1] - y * sign * ARM,
                z * sign,
            ],
        )
    });
    points.sort_by(|a, b| a.2[2].total_cmp(&b.2[2]));
    points
}

fn hit(points: &[(usize, bool, [f32; 3]); 6], mouse: [f32; 2]) -> Option<(usize, bool)> {
    points
        .iter()
        .rev()
        .find_map(|&(axis, positive, [x, y, _])| {
            ((mouse[0] - x).powi(2) + (mouse[1] - y).powi(2) <= HOVER_RADIUS.powi(2))
                .then_some((axis, positive))
        })
}

/// Receives the world axes expressed in camera coordinates.
pub fn draw(
    ui: &Ui,
    min: [f32; 2],
    size: [f32; 2],
    axes: [[f32; 3]; 3],
    state: &mut State,
) -> Response {
    let [origin, _] = bounds(min, size);
    let center = [origin[0] + SIZE * 0.5, origin[1] + SIZE * 0.5];
    ui.set_cursor_screen_pos(origin);
    ui.set_next_item_allow_overlap();
    ui.invisible_button("Viewport orientation", [SIZE; 2]);
    let hovered = ui.is_item_hovered();
    let active = ui.is_item_active();
    let points = points(axes, center);
    let target = hovered.then(|| hit(&points, ui.io().mouse_pos())).flatten();
    let mut response = Response::default();
    if hovered && ui.is_mouse_clicked(MouseButton::Left) {
        state.started = true;
        state.pressed = target;
        state.dragged = false;
    }
    if active && ui.is_mouse_dragging(MouseButton::Left) {
        state.dragged = true;
        response.orbit = Some(ui.io().mouse_delta());
    }
    if state.started && ui.is_mouse_released(MouseButton::Left) {
        if !state.dragged && state.pressed == target {
            response.axis = target;
        }
        state.started = false;
    }
    if !ui.is_mouse_down(MouseButton::Left) {
        state.started = false;
    }

    let draw = ui.get_window_draw_list();
    if hovered || active {
        draw.add_circle(
            center,
            ARM + HOVER_RADIUS + OUTLINE * 0.5,
            [0.15, 0.15, 0.15, 0.5],
        )
        .filled(true)
        .build();
    }
    for &(axis, positive, [x, y, _]) in &points {
        let color = COLORS[axis];
        if positive {
            draw.add_line(center, [x, y], color).thickness(2.0).build();
        }
        let highlighted = target == Some((axis, positive));
        let radius = if highlighted { HOVER_RADIUS } else { RADIUS };
        draw.add_circle(
            [x, y],
            radius,
            if positive {
                color
            } else {
                [color[0] * 0.45, color[1] * 0.45, color[2] * 0.45, 1.0]
            },
        )
        .filled(true)
        .build();
        draw.add_circle([x, y], radius, if highlighted { [1.0; 4] } else { color })
            .thickness(OUTLINE)
            .build();
        if positive || highlighted {
            let label = ["X", "Y", "Z"][axis];
            let text = ui.calc_text_size(label);
            draw.add_text(
                [x - text[0] * 0.5, y - text[1] * 0.5],
                if highlighted {
                    [1.0; 4]
                } else {
                    [0.05, 0.05, 0.05, 1.0]
                },
                label,
            );
        }
    }
    response
}

#[cfg(test)]
mod tests {
    #[test]
    fn axes_project_in_camera_space_and_clicks_choose_the_nearest_ball() {
        let front = super::points(
            [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            [50.0; 2],
        );
        assert_eq!(super::hit(&front, [83.0, 50.0]), Some((0, true)));
        assert_eq!(super::hit(&front, [50.0, 17.0]), Some((1, true)));
        assert_eq!(super::hit(&front, [50.0, 50.0]), Some((2, true)));
        let back = super::points(
            [[-1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, -1.0]],
            [50.0; 2],
        );
        assert_eq!(super::hit(&back, [50.0, 50.0]), Some((2, false)));
        assert_eq!(super::hit(&back, [83.0, 50.0]), Some((0, false)));
        assert_eq!(super::hit(&front, [100.0; 2]), None);
    }

    #[test]
    fn clicking_a_ball_aligns_but_dragging_orbits_without_snapping() {
        use dear_imgui_rs::{Condition, Context, MouseButton};
        let _lock = crate::TEST_LOCK.lock().unwrap();
        let mut context = Context::create();
        context.set_ini_filename(None::<&str>).unwrap();
        context
            .font_atlas()
            .try_claim_legacy_renderer()
            .unwrap()
            .build();
        context.io_mut().set_display_size([640.0, 360.0]);
        context.io_mut().set_delta_time(1.0 / 60.0);
        let mut state = super::State::default();
        fn frame(
            context: &mut Context,
            state: &mut super::State,
            point: [f32; 2],
            down: bool,
        ) -> super::Response {
            context.io_mut().add_mouse_pos_event(point);
            context
                .io_mut()
                .add_mouse_button_event(MouseButton::Left, down);
            let ui = context.frame();
            let mut response = super::Response::default();
            ui.window("Orientation test")
                .position([0.0; 2], Condition::Always)
                .size([640.0, 360.0], Condition::Always)
                .build(|| {
                    response = super::draw(
                        ui,
                        [0.0, 30.0],
                        [640.0, 300.0],
                        [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                        state,
                    );
                });
            context.end_frame();
            response
        }
        let ball = [613.0, 90.0];
        frame(&mut context, &mut state, ball, false);
        frame(&mut context, &mut state, ball, false);
        assert!(frame(&mut context, &mut state, ball, true).axis.is_none());
        assert_eq!(
            frame(&mut context, &mut state, ball, false).axis,
            Some((0, true))
        );
        frame(&mut context, &mut state, ball, true);
        let dragged = [600.0, 105.0];
        assert!(
            frame(&mut context, &mut state, dragged, true)
                .orbit
                .is_some()
        );
        assert!(
            frame(&mut context, &mut state, dragged, false)
                .axis
                .is_none()
        );
    }
}
