//! Image placement, preview navigation, and editor overlays.

#[derive(Clone, Copy)]
pub struct Image {
    pub size: [f32; 2],
    pub texture: dear_imgui_rs::TextureId,
    pub nearest: bool,
}

#[derive(Clone, Copy)]
pub struct View {
    pub pan: [f32; 2],
    pub zoom: f32,
}

impl Default for View {
    fn default() -> Self {
        Self {
            pan: [0.0; 2],
            zoom: 1.0,
        }
    }
}

impl View {
    pub fn zoom_at(&mut self, wheel: f32, anchor: [f32; 2]) {
        if wheel == 0.0 {
            return;
        }

        let zoom = (self.zoom * (wheel * 0.15).exp()).clamp(0.01, 100.0);
        let ratio = zoom / self.zoom;
        for axis in 0..2 {
            self.pan[axis] = anchor[axis] + (self.pan[axis] - anchor[axis]) * ratio;
        }
        self.zoom = zoom;
    }
}

pub fn draw(
    ui: &dear_imgui_rs::Ui,
    preview: Image,
    available: [f32; 2],
    margin: Option<(u32, u32)>,
) {
    let (_, size, min, max) = placement(ui.cursor_screen_pos(), available, preview.size);

    ui.set_cursor_screen_pos(min);

    set_sampler(&ui.get_window_draw_list(), preview.nearest);

    ui.image_config(preview.texture, size)
        .uv0([0.0, 1.0])
        .uv1([1.0, 0.0])
        .build();
    ui.get_window_draw_list().set_sampler_linear();

    if let Some(margin) = margin {
        draw_margin(
            ui,
            &ui.get_window_draw_list(),
            min,
            max,
            preview.size,
            margin,
        );
    }

    ui.get_window_draw_list()
        .add_rect(min, max, ui.style_color(dear_imgui_rs::StyleColor::Border))
        .thickness(1.0)
        .build();
}

pub fn draw_navigable(
    ui: &dear_imgui_rs::Ui,
    preview: Image,
    available: [f32; 2],
    margin: Option<(u32, u32)>,
    view: &mut View,
) -> bool {
    let origin = ui.cursor_screen_pos();
    let size = [available[0].max(1.0), available[1].max(1.0)];
    let max = [origin[0] + size[0], origin[1] + size[1]];
    let center = [origin[0] + size[0] * 0.5, origin[1] + size[1] * 0.5];
    let (scale, fitted, _, _) = placement(origin, size, preview.size);
    ui.invisible_button_options(
        "Preview canvas.",
        size,
        dear_imgui_rs::InvisibleButtonOptions::new()
            .flags(dear_imgui_rs::ButtonFlags::ALLOW_OVERLAP)
            .mouse_buttons(dear_imgui_rs::InvisibleButtonMouseButtons::MIDDLE),
    );
    let mouse = ui.io().mouse_pos();
    let over_tools = mouse[0] >= origin[0] + 8.0
        && mouse[0] <= origin[0] + 16.0 + ui.frame_height() * 2.0
        && mouse[1] >= origin[1] + 8.0
        && mouse[1] <= origin[1] + 8.0 + ui.frame_height();
    let hovered = ui.is_item_hovered() && !over_tools;
    if !over_tools
        && ui.is_item_active()
        && ui.is_mouse_dragging(dear_imgui_rs::MouseButton::Middle)
    {
        let delta = ui.io().mouse_delta();
        for axis in 0..2 {
            view.pan[axis] += delta[axis] / scale;
        }
        ui.set_mouse_cursor(Some(dear_imgui_rs::MouseCursor::ResizeAll));
    }
    if hovered {
        let mouse = ui.io().mouse_pos();
        view.zoom_at(
            ui.io().mouse_wheel(),
            [
                (mouse[0] - center[0]) / scale,
                (mouse[1] - center[1]) / scale,
            ],
        );
    }
    let min = [
        center[0] + view.pan[0] * scale - fitted[0] * view.zoom * 0.5,
        center[1] + view.pan[1] * scale - fitted[1] * view.zoom * 0.5,
    ];
    let image_max = [
        min[0] + fitted[0] * view.zoom,
        min[1] + fitted[1] * view.zoom,
    ];
    let draw = ui.get_window_draw_list();
    let _clip = draw.push_clip_rect(origin, max, true);
    set_sampler(&draw, preview.nearest);
    draw.add_image(
        preview.texture,
        min,
        image_max,
        [0.0, 1.0],
        [1.0, 0.0],
        [1.0; 4],
    );
    draw.set_sampler_linear();
    if let Some(margin) = margin {
        draw_margin(ui, &draw, min, image_max, preview.size, margin);
    }
    draw.add_rect(
        min,
        image_max,
        ui.style_color(dear_imgui_rs::StyleColor::Border),
    )
    .thickness(1.0)
    .build();
    hovered && ui.is_mouse_clicked(dear_imgui_rs::MouseButton::Left)
}

fn placement(
    origin: [f32; 2],
    available: [f32; 2],
    source: [f32; 2],
) -> (f32, [f32; 2], [f32; 2], [f32; 2]) {
    let available = [available[0].max(1.0), available[1].max(1.0)];
    let scale = (available[0] / source[0]).min(available[1] / source[1]);
    let size = [source[0] * scale, source[1] * scale];
    let min = [
        origin[0] + (available[0] - size[0]) * 0.5,
        origin[1] + (available[1] - size[1]) * 0.5,
    ];

    (scale, size, min, [min[0] + size[0], min[1] + size[1]])
}

pub fn camera_mask(ui: &dear_imgui_rs::Ui, min: [f32; 2], size: [f32; 2], camera_aspect: f32) {
    let size = [size[0].max(1.0), size[1].max(1.0)];
    let max = [min[0] + size[0], min[1] + size[1]];
    let draw = ui.get_window_draw_list();
    let _clip = draw.push_clip_rect(min, max, true);
    let [rect_min, rect_max] = camera_frame(min, size, camera_aspect);

    for [shade_min, shade_max] in [
        [min, [max[0], rect_min[1]]],
        [[min[0], rect_max[1]], max],
        [[min[0], rect_min[1]], [rect_min[0], rect_max[1]]],
        [[rect_max[0], rect_min[1]], [max[0], rect_max[1]]],
    ] {
        draw.add_rect(shade_min, shade_max, [0.0, 0.0, 0.0, 0.25])
            .filled(true)
            .build();
    }

    draw.add_rect(
        rect_min,
        rect_max,
        ui.style_color(dear_imgui_rs::StyleColor::Border),
    )
    .thickness(1.0)
    .build();
}

pub fn camera_frame(min: [f32; 2], size: [f32; 2], camera_aspect: f32) -> [[f32; 2]; 2] {
    let editor_aspect = size[0].max(1.0) / size[1].max(1.0);
    let (width, height) = if editor_aspect >= camera_aspect {
        (size[0] * camera_aspect / editor_aspect, size[1])
    } else {
        (size[0], size[1] * editor_aspect / camera_aspect)
    };
    let left = min[0] + (size[0] - width) * 0.5;
    let top = min[1] + (size[1] - height) * 0.5;

    [[left, top], [left + width, top + height]]
}

pub fn project_outline(ui: &dear_imgui_rs::Ui, min: [f32; 2], size: [f32; 2], aspect: f32) {
    let [rect_min, rect_max] = camera_frame(min, size, aspect);

    ui.get_window_draw_list()
        .add_rect(
            rect_min,
            rect_max,
            ui.style_color(dear_imgui_rs::StyleColor::Border),
        )
        .thickness(1.0)
        .build();
}

fn margin_color(ui: &dear_imgui_rs::Ui) -> [f32; 4] {
    let mut color = ui.style_color(dear_imgui_rs::StyleColor::CheckMark);

    color[3] = 0.7;

    color
}

fn draw_margin(
    ui: &dear_imgui_rs::Ui,
    draw: &dear_imgui_rs::DrawListMut<'_>,
    min: [f32; 2],
    max: [f32; 2],
    source: [f32; 2],
    margin: (u32, u32),
) {
    let inset = [
        (max[0] - min[0]) * margin.0 as f32 / source[0].max(1.0),
        (max[1] - min[1]) * margin.1 as f32 / source[1].max(1.0),
    ];
    let a = [min[0] + inset[0], min[1] + inset[1]];
    let b = [max[0] - inset[0], max[1] - inset[1]];

    draw.add_rect(a, b, [0.0, 0.0, 0.0, 0.7])
        .thickness(3.0)
        .build();

    draw.add_rect(a, b, margin_color(ui)).thickness(1.0).build();
}

fn set_sampler(draw_list: &dear_imgui_rs::DrawListMut<'_>, nearest: bool) {
    match nearest {
        true => draw_list.set_sampler_nearest(),
        false => draw_list.set_sampler_linear(),
    }
}

#[cfg(test)]
mod tests {
    use super::placement;

    #[test]
    fn fitted_video_preserves_aspect_and_stays_above_the_footer() {
        let origin = [10.0, 20.0];
        let viewport = [800.0, 450.0];
        for source in [[1920.0, 1080.0], [1080.0, 1920.0], [1000.0, 1000.0]] {
            let (_, size, min, max) = placement(origin, viewport, source);
            assert!((size[0] / size[1] - source[0] / source[1]).abs() < 0.001);
            assert!(min[0] >= origin[0] && min[1] >= origin[1]);
            assert!(max[0] <= origin[0] + viewport[0]);
            assert!(max[1] <= origin[1] + viewport[1]);
        }
    }
}

pub type Quad = [[f32; 2]; 4];

pub struct Editor2D {
    pub image: Image,
    pub bounds: [[f32; 2]; 2],
    pub display_scale: f32,
    pub view: View,
    pub rotation: f32,
    pub background: [f32; 4],
    pub axes: [bool; 2],
    pub project: Quad,
    pub camera: Option<Quad>,
    pub selection: Option<Quad>,
    pub margin: Option<(u32, u32)>,
}

pub fn draw_editor_2d(ui: &dear_imgui_rs::Ui, props: &Editor2D) {
    let [min, max] = props.bounds;
    let center = [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5];
    let preview = props.image;
    let display_scale = props.display_scale;
    let View { pan, zoom } = props.view;
    let background = props.background;
    let draw_list = ui.get_window_draw_list();
    let _clip = draw_list.push_clip_rect(min, max, true);

    draw_list
        .add_rect(min, max, background)
        .filled(true)
        .build();

    set_sampler(&draw_list, preview.nearest);
    draw_list.add_image(preview.texture, min, max, [0.0, 1.0], [1.0, 0.0], [1.0; 4]);
    draw_list.set_sampler_linear();

    let origin = [
        center[0] + pan[0] * display_scale,
        center[1] + pan[1] * display_scale,
    ];

    let length = (max[0] - min[0]).hypot(max[1] - min[1])
        + (origin[0] - center[0]).hypot(origin[1] - center[1]);
    let (sin, cos) = props.rotation.sin_cos();
    for (enabled, direction, color) in [
        (props.axes[0], [cos, sin], [1.0, 0.3, 0.3, 1.0]),
        (props.axes[1], [-sin, cos], [0.3, 1.0, 0.3, 1.0]),
    ] {
        if enabled {
            draw_list
                .add_line(
                    [
                        origin[0] - direction[0] * length,
                        origin[1] - direction[1] * length,
                    ],
                    [
                        origin[0] + direction[0] * length,
                        origin[1] + direction[1] * length,
                    ],
                    color,
                )
                .thickness(1.0)
                .build();
        }
    }

    draw_world_outline(
        &draw_list,
        Some(props.project),
        center,
        display_scale,
        pan,
        zoom,
        props.rotation,
        ui.style_color(dear_imgui_rs::StyleColor::Border),
        1.0,
    );

    if let Some(margin) = props.margin {
        let mut points = props.project;

        for point in &mut points {
            point[0] += if point[0] < 0.0 {
                margin.0 as f32
            } else {
                -(margin.0 as f32)
            };

            point[1] += if point[1] < 0.0 {
                margin.1 as f32
            } else {
                -(margin.1 as f32)
            };
        }

        let accent = margin_color(ui);

        draw_world_outline(
            &draw_list,
            Some(points),
            center,
            display_scale,
            pan,
            zoom,
            props.rotation,
            [0.0, 0.0, 0.0, 0.7],
            3.0,
        );

        draw_world_outline(
            &draw_list,
            Some(points),
            center,
            display_scale,
            pan,
            zoom,
            props.rotation,
            accent,
            1.0,
        );
    }

    if props.camera.is_some() {
        draw_world_outline(
            &draw_list,
            props.camera,
            center,
            display_scale,
            pan,
            zoom,
            props.rotation,
            ui.style_color(dear_imgui_rs::StyleColor::Border),
            1.0,
        );
    }

    let selection = props.selection;

    draw_world_outline(
        &draw_list,
        selection,
        center,
        display_scale,
        pan,
        zoom,
        props.rotation,
        [0.0, 0.0, 0.0, 1.0],
        3.0,
    );

    draw_world_outline(
        &draw_list,
        selection,
        center,
        display_scale,
        pan,
        zoom,
        props.rotation,
        [1.0, 1.0, 1.0, 1.0],
        1.0,
    );
}

pub fn draw_editor_3d(
    ui: &dear_imgui_rs::Ui,
    preview: Image,
    available: [f32; 2],
) -> (bool, Option<[f32; 2]>) {
    let size = [available[0].max(1.0), available[1].max(1.0)];

    ui.invisible_button_options(
        "Editor 3D canvas.",
        size,
        dear_imgui_rs::InvisibleButtonOptions::new()
            .flags(dear_imgui_rs::ButtonFlags::ALLOW_OVERLAP)
            .mouse_buttons(dear_imgui_rs::InvisibleButtonMouseButtons::RIGHT),
    );

    let hovered = ui.is_item_hovered();
    let active = ui.is_item_active();
    let min = ui.item_rect_min();
    let max = ui.item_rect_max();

    set_sampler(&ui.get_window_draw_list(), preview.nearest);
    ui.get_window_draw_list().add_image(
        preview.texture,
        min,
        max,
        [0.0, 1.0],
        [1.0, 0.0],
        [1.0; 4],
    );
    ui.get_window_draw_list().set_sampler_linear();

    let mouse = ui.io().mouse_pos();
    let over_camera = mouse[0] >= min[0] + 8.0
        && mouse[0] <= min[0] + 24.0 + ui.frame_height() * 3.0
        && mouse[1] >= min[1] + 8.0
        && mouse[1] <= min[1] + 8.0 + ui.frame_height();
    let over_controls = mouse[0] >= max[0] - 112.0
        && mouse[0] <= max[0]
        && mouse[1] >= min[1] + 8.0
        && mouse[1] <= min[1] + 8.0 + ui.frame_height();
    let clicked = (hovered
        && !over_camera
        && !over_controls
        && ui.is_mouse_clicked(dear_imgui_rs::MouseButton::Left))
    .then(|| {
        [
            mouse[0] - (min[0] + max[0]) * 0.5,
            mouse[1] - (min[1] + max[1]) * 0.5,
        ]
    });

    (active, clicked)
}

#[allow(clippy::too_many_arguments)]
fn draw_world_outline(
    draw_list: &dear_imgui_rs::DrawListMut<'_>,
    points: Option<Quad>,
    center: [f32; 2],
    display_scale: f32,
    pan: [f32; 2],
    zoom: f32,
    rotation: f32,
    color: [f32; 4],
    thickness: f32,
) {
    let Some(points) = points else {
        return;
    };

    let project = |point: [f32; 2]| {
        let (sin, cos) = rotation.sin_cos();
        let point = [
            point[0] * cos - point[1] * sin,
            point[0] * sin + point[1] * cos,
        ];
        [
            center[0] + (pan[0] + point[0] * zoom) * display_scale,
            center[1] + (pan[1] + point[1] * zoom) * display_scale,
        ]
    };

    for index in 0..4 {
        draw_list
            .add_line(
                project(points[index]),
                project(points[(index + 1) % 4]),
                color,
            )
            .thickness(thickness)
            .build();
    }
}

fn axes(ui: &dear_imgui_rs::Ui, min: [f32; 2], size: [f32; 2], enabled: &[bool]) -> Option<usize> {
    ui.set_cursor_screen_pos([
        min[0] + size[0] - (32.0 * enabled.len() as f32 + 16.0),
        min[1] + 8.0,
    ]);
    let mut clicked = None;
    for (axis, (&enabled, (label, color))) in enabled
        .iter()
        .zip([
            ("X", [1.0, 0.3, 0.3, 1.0]),
            ("Y", [0.3, 1.0, 0.3, 1.0]),
            ("Z", [0.35, 0.55, 1.0, 1.0]),
        ])
        .enumerate()
    {
        if axis > 0 {
            ui.same_line();
        }
        ui.set_next_item_allow_overlap();
        let color = if enabled {
            color
        } else {
            ui.style_color(dear_imgui_rs::StyleColor::TextDisabled)
        };
        let _color = ui.push_style_color(dear_imgui_rs::StyleColor::Text, color);
        if crate::controls::button_sized(ui, label, [32.0, ui.frame_height()]) {
            clicked = Some(axis);
        }
    }
    clicked
}

pub fn margin(ui: &dear_imgui_rs::Ui, bounds: [[f32; 2]; 2], source: [f32; 2], margin: (u32, u32)) {
    draw_margin(
        ui,
        &ui.get_window_draw_list(),
        bounds[0],
        bounds[1],
        source,
        margin,
    );
}

pub fn wrap_target(mouse: [f32; 2], min: [f32; 2], max: [f32; 2]) -> Option<[f32; 2]> {
    const EDGE: f32 = 2.0;

    const INSET: f32 = 6.0;

    let mut target = mouse;

    if mouse[0] <= min[0] + EDGE {
        target[0] = max[0] - INSET;
    } else if mouse[0] >= max[0] - EDGE {
        target[0] = min[0] + INSET;
    }

    if mouse[1] <= min[1] + EDGE {
        target[1] = max[1] - INSET;
    } else if mouse[1] >= max[1] - EDGE {
        target[1] = min[1] + INSET;
    }

    (target != mouse).then_some(target)
}

#[derive(Default)]
pub struct Tools {
    pub camera: bool,
    pub reset: bool,
    pub margin: bool,
    pub axis: Option<usize>,
}

pub fn margin_button(ui: &dear_imgui_rs::Ui, visible: bool, shortcut: &str) -> bool {
    let clicked = crate::controls::icon_toggle(ui, "M", visible);
    if ui.is_item_hovered() {
        ui.tooltip_text(format!("Toggle margin [{shortcut}]"));
    }
    clicked
}

pub fn tools(
    ui: &dear_imgui_rs::Ui,
    min: [f32; 2],
    size: [f32; 2],
    camera_view: bool,
    margin: bool,
    enabled_axes: &[bool],
    shortcuts: [&str; 3],
) -> Tools {
    ui.set_cursor_screen_pos([min[0] + 8.0, min[1] + 8.0]);
    let camera = crate::controls::icon_toggle(ui, crate::icons::VIDEO_CAMERA, camera_view);
    if ui.is_item_hovered() {
        ui.tooltip_text(format!(
            "{} [{}]",
            if camera_view {
                "Use the free editor camera"
            } else {
                "Use the canvas camera"
            },
            shortcuts[0]
        ));
    }
    ui.same_line();
    let reset = crate::controls::text_button(ui, crate::icons::RESET, [ui.frame_height(); 2]);
    if ui.is_item_hovered() {
        ui.tooltip_text(format!("Reset camera transform [{}]", shortcuts[1]));
    }
    ui.same_line();
    let margin = margin_button(ui, margin, shortcuts[2]);
    let axis = axes(ui, min, size, enabled_axes);
    Tools {
        camera,
        reset,
        margin,
        axis,
    }
}

pub fn preview_tools(
    ui: &dear_imgui_rs::Ui,
    min: [f32; 2],
    view: &mut View,
    margin: bool,
    shortcuts: [&str; 2],
) -> bool {
    ui.set_cursor_screen_pos([min[0] + 8.0, min[1] + 8.0]);
    ui.set_next_item_allow_overlap();
    if crate::controls::text_button(ui, crate::icons::RESET, [ui.frame_height(); 2]) {
        *view = View::default();
    }
    if ui.is_item_hovered() {
        ui.tooltip_text(format!("Fit canvas [{}]", shortcuts[0]));
    }
    ui.same_line();
    margin_button(ui, margin, shortcuts[1])
}

#[cfg(test)]
mod frame_tests {
    use super::camera_frame;

    #[test]
    fn camera_frame_keeps_the_canvas_aspect_inside_a_dynamic_view() {
        let wide_camera = camera_frame([10.0, 20.0], [1600.0, 1000.0], 16.0 / 9.0);

        assert_eq!([wide_camera[0][0], wide_camera[1][0]], [10.0, 1610.0]);
        assert!((wide_camera[0][1] - 70.0).abs() < 0.001);
        assert!((wide_camera[1][1] - 970.0).abs() < 0.001);

        assert_eq!(
            camera_frame([0.0, 0.0], [2000.0, 1000.0], 1.0),
            [[500.0, 0.0], [1500.0, 1000.0]]
        );
    }
}

#[cfg(test)]
mod navigation_tests {
    #[test]
    fn preview_zoom_preserves_the_mouse_anchor_and_fit_resets_the_view() {
        let mut view = super::View::default();
        view.pan = [20.0, -10.0];
        let anchor = [80.0, 40.0];
        let point = [anchor[0] - view.pan[0], anchor[1] - view.pan[1]];
        view.zoom_at(3.0, anchor);
        for axis in 0..2 {
            assert!((view.pan[axis] + point[axis] * view.zoom - anchor[axis]).abs() < 0.001);
        }
        view.zoom_at(1000.0, anchor);
        assert_eq!(view.zoom, 100.0);
        view = super::View::default();
        assert_eq!(view.pan, [0.0; 2]);
        assert_eq!(view.zoom, 1.0);
        view.zoom_at(-1.0, anchor);
        assert!(view.zoom < 1.0);
    }

    #[test]
    fn preview_overlay_controls_and_canvas_navigation_work_together() {
        use super::*;
        use dear_imgui_rs::{Condition, Context, MouseButton, TextureId};
        let _lock = crate::TEST_LOCK.lock().unwrap();
        let mut context = Context::create();
        context.set_ini_filename(None::<&str>).unwrap();
        context
            .font_atlas()
            .try_claim_legacy_renderer()
            .unwrap()
            .build();
        unsafe extern "C" fn sampler(
            _: *const dear_imgui_rs::sys::ImDrawList,
            _: *const dear_imgui_rs::sys::ImDrawCmd,
        ) {
        }
        // The test records draw commands without rendering; this static callback owns no data.
        unsafe {
            context
                .platform_io_mut()
                .set_draw_callback_set_sampler_linear_raw(Some(sampler));
        }
        context.io_mut().set_display_size([640.0, 360.0]);
        context.io_mut().set_delta_time(1.0 / 60.0);
        fn frame(
            context: &mut Context,
            view: &mut View,
            point: [f32; 2],
            buttons: [bool; 2],
            wheel: f32,
        ) -> ([[f32; 2]; 2], bool) {
            let io = context.io_mut();
            io.add_mouse_pos_event(point);
            io.add_mouse_button_event(MouseButton::Left, buttons[0]);
            io.add_mouse_button_event(MouseButton::Middle, buttons[1]);
            io.add_mouse_wheel_event([0.0, wheel]);
            let ui = context.frame();
            let mut targets = [[0.0; 2]; 2];
            let mut margin = false;
            ui.window("Preview navigation test")
                .position([0.0; 2], Condition::Always)
                .size([640.0, 360.0], Condition::Always)
                .build(|| {
                    let origin = ui.cursor_screen_pos();
                    targets[0] = [
                        origin[0] + 8.0 + ui.frame_height() * 0.5,
                        origin[1] + 8.0 + ui.frame_height() * 0.5,
                    ];
                    draw_navigable(
                        ui,
                        Image {
                            size: [1920.0, 1080.0],
                            texture: TextureId::new(1),
                            nearest: false,
                        },
                        [624.0, 300.0],
                        None,
                        view,
                    );
                    margin = preview_tools(ui, origin, view, false, ["0", "M"]);
                    let min = ui.item_rect_min();
                    let max = ui.item_rect_max();
                    targets[1] = [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5];
                });
            context.end_frame();
            (targets, margin)
        }
        let mut view = View {
            pan: [20.0, -10.0],
            zoom: 1.5,
        };
        let ([reset, margin], _) = frame(&mut context, &mut view, [0.0; 2], [false; 2], 0.0);
        frame(&mut context, &mut view, reset, [false; 2], 0.0);
        frame(&mut context, &mut view, reset, [true, false], 0.0);
        frame(&mut context, &mut view, reset, [false; 2], 0.0);
        assert_eq!(view.pan, [0.0; 2]);
        assert_eq!(view.zoom, 1.0);
        frame(&mut context, &mut view, margin, [false; 2], 0.0);
        frame(&mut context, &mut view, margin, [true, false], 0.0);
        assert!(frame(&mut context, &mut view, margin, [false; 2], 0.0).1);
        let point = [300.0, 180.0];
        frame(&mut context, &mut view, point, [false; 2], 0.0);
        frame(&mut context, &mut view, point, [false, true], 0.0);
        frame(
            &mut context,
            &mut view,
            [point[0] + 20.0, point[1]],
            [false, true],
            0.0,
        );
        assert!(view.pan[0] > 0.0);
        frame(&mut context, &mut view, point, [false; 2], 0.0);
        frame(&mut context, &mut view, point, [false; 2], 1.0);
        frame(&mut context, &mut view, point, [false; 2], 0.0);
        assert!(view.zoom > 1.0);
    }

    #[test]
    fn pointer_wraps_inside_the_opposite_edge() {
        assert_eq!(super::wrap_target([50.0, 50.0], [0.0; 2], [100.0; 2]), None);
        assert_eq!(
            super::wrap_target([0.0, 50.0], [0.0; 2], [100.0; 2]),
            Some([94.0, 50.0])
        );
        assert_eq!(
            super::wrap_target([100.0, 100.0], [0.0; 2], [100.0; 2]),
            Some([6.0, 6.0])
        );
    }
}
