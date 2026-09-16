use kinematic::prelude::{
    draw_image_2d, draw_image_3d, draw_latex_2d, draw_latex_3d, draw_line_3d, draw_text_2d,
    draw_text_3d,
};

#[test]
fn complex_draw_helpers_are_public() {
    let _ = draw_text_2d;
    let _ = draw_latex_2d;
    let _ = draw_image_2d;
    let _ = draw_text_3d;
    let _ = draw_latex_3d;
    let _ = draw_image_3d;
    let _ = draw_line_3d;
}
