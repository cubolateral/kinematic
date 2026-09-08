use super::Canvases;
use crate::prelude::*;
use crate::renderer::target::{Target, reset_gl};
use glow::HasContext;

/// Exercises real GPU interop; run with SDL_VIDEODRIVER=offscreen when available.
#[test]
#[ignore = "Requires an SDL OpenGL 3.3 context and FFmpeg."]
fn graphics_canvas_projection_alpha_orientation() {
    let sdl = sdl3::init().unwrap();
    let video = sdl.video().unwrap();
    video.gl_attr().set_context_version(3, 3);
    video
        .gl_attr()
        .set_context_profile(sdl3::video::GLProfile::Core);
    let window = video
        .window("Canvas test", 64, 64)
        .hidden()
        .opengl()
        .build()
        .unwrap();
    let context = window.gl_create_context().unwrap();
    window.gl_make_current(&context).unwrap();
    let load = |name: &str| {
        video
            .gl_get_proc_address(name)
            .map(|p| p as *const std::ffi::c_void)
            .unwrap_or(std::ptr::null())
    };
    let gl = std::rc::Rc::new(unsafe { glow::Context::from_loader_function(load) });
    let three_gl = unsafe { glow::Context::from_loader_function(load) };
    let three = three_d::Context::from_gl_context(std::sync::Arc::new(three_gl)).unwrap();
    let interface = skia_safe::gpu::gl::Interface::new_load_with(|name| {
        if name == "eglGetCurrentDisplay" {
            std::ptr::null()
        } else {
            load(name)
        }
    })
    .unwrap();
    let mut skia = skia_safe::gpu::direct_contexts::make_gl(interface, None).unwrap();
    let mut renderer = Canvases::new(three, &gl);
    let mut output = Target::new((64, 64), false, &mut skia, &gl).unwrap();
    let mut scene = Scene::new_with_resolution((64, 64));
    let overlay = scene.get_world_2d();
    let top = rect()
        .size(vec2(64.0, 32.0))
        .position(vec2(0.0, -16.0))
        .fill(Color::new(0.5, 0.25, 0.0, 0.5))
        .build(&mut scene);
    let bottom = rect()
        .size(vec2(64.0, 32.0))
        .position(vec2(0.0, 16.0))
        .fill(Color::BLUE)
        .build(&mut scene);
    overlay.add(&top);
    overlay.add(&bottom);
    let world = scene.get_world_3d();
    let camera = camera_3d()
        .position(vec3(0.0, 0.0, 2.0))
        .fov(std::f32::consts::FRAC_PI_2)
        .build(&mut scene);
    world.add(&camera);
    world.set_camera(&camera);
    let screen = projection()
        .source(&overlay)
        .size(vec2(4.0, 4.0))
        .build(&mut scene);
    world.add(&screen);
    scene.get_root().view_2d(false).immediate();
    scene.update(0.0);
    renderer.render(&scene, &mut output, &mut skia).unwrap();
    let source_texture = renderer.targets[&overlay.get_texture()].texture();
    let first = read(&gl, &output);
    assert_pixel(&first, 32, 48, [64, 32, 0, 255]);
    assert_pixel(
        &read(&gl, &renderer.targets[&world.get_texture()]),
        32,
        48,
        [64, 32, 0, 128],
    );
    assert_pixel(&first, 32, 16, [0, 0, 255, 255]);
    renderer.render(&scene, &mut output, &mut skia).unwrap();
    assert_eq!(read(&gl, &output), first);
    verify_export(&gl, &output, &first);
    assert_eq!(
        renderer.targets[&overlay.get_texture()].texture(),
        source_texture
    );
    world.clear(Color::BLUE).play();
    scene.update(0.0);
    renderer.render(&scene, &mut output, &mut skia).unwrap();
    assert_pixel(&read(&gl, &output), 32, 48, [64, 32, 127, 255]);
    screen.remove();
    scene.update(1.0);
    renderer.render(&scene, &mut output, &mut skia).unwrap();
    assert_pixel(&read(&gl, &output), 32, 48, [0, 0, 255, 255]);

    let cube = cuboid().size(vec3(0.8, 0.8, 0.8)).build(&mut scene);
    let ball = sphere()
        .radius(0.3)
        .position(vec3(1.0, 0.0, 0.0))
        .unlit(true)
        .albedo(Color::GREEN)
        .build(&mut scene);
    let plane = plane()
        .size(vec2(0.5, 0.5))
        .position(vec3(-1.0, 0.0, 0.0))
        .unlit(true)
        .albedo(Color::RED)
        .build(&mut scene);
    world.add(&cube);
    world.add(&ball);
    world.add(&plane);
    scene.update(1.0);
    renderer.render(&scene, &mut output, &mut skia).unwrap();
    let pixels = read(&gl, &output);
    assert_ne!(
        &pixels[(32 * 64 + 32) * 4..(32 * 64 + 32) * 4 + 4],
        &[0, 0, 255, 255]
    );
    assert_pixel(&pixels, 48, 32, [0, 255, 0, 255]);
    assert_pixel(&pixels, 16, 32, [255, 0, 0, 255]);
    reset_gl(&gl, (64, 64));
    assert_eq!(unsafe { gl.get_error() }, glow::NO_ERROR);
    let allocations: Vec<_> = renderer
        .targets
        .values()
        .map(|target| (target.texture(), target.framebuffer()))
        .collect();
    drop(renderer);
    for (texture, framebuffer) in allocations {
        assert!(!unsafe { gl.is_texture(texture) });
        assert!(!unsafe { gl.is_framebuffer(framebuffer) });
    }
    drop(output);
    skia.flush_and_submit();
}

fn read(gl: &glow::Context, target: &Target) -> Vec<u8> {
    let mut pixels = vec![0; 64 * 64 * 4];
    unsafe {
        gl.bind_buffer(glow::PIXEL_PACK_BUFFER, None);
        gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(target.framebuffer()));
        gl.read_pixels(
            0,
            0,
            64,
            64,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            glow::PixelPackData::Slice(Some(&mut pixels)),
        );
    }
    pixels
}
fn assert_pixel(pixels: &[u8], x: usize, y: usize, expected: [u8; 4]) {
    let actual = &pixels[(y * 64 + x) * 4..(y * 64 + x) * 4 + 4];
    assert!(
        actual.iter().zip(expected).all(|(a, b)| a.abs_diff(b) <= 2),
        "Pixel ({x}, {y}): {actual:?}, expected {expected:?}."
    );
}

fn verify_export(gl: &glow::Context, target: &Target, preview: &[u8]) {
    use crate::renderer::{Encoder, Export, FrameResult, Renderer};
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "kinematic-canvas-{}-{nonce}.mp4",
        std::process::id()
    ));
    let mut exporter = Renderer::new(target.size);
    exporter.export = Some(Export {
        encoder: Encoder::new(&path, target.size, 30, true).unwrap(),
        output_path: path.clone(),
        frame_count: 1,
        frame_index: 0,
        fps: 30,
    });
    assert!(matches!(
        exporter
            .process_frame(gl, target.framebuffer(), target.size)
            .unwrap(),
        FrameResult::Finished
    ));
    exporter.shutdown(gl);
    let decoded = std::process::Command::new("ffmpeg")
        .args(["-v", "error", "-i"])
        .arg(&path)
        .args(["-frames:v", "1", "-f", "rawvideo", "-pix_fmt", "rgba", "-"])
        .output()
        .unwrap();
    std::fs::remove_file(path).unwrap();
    assert!(decoded.status.success());
    assert_eq!(decoded.stdout.len(), preview.len());
    for (x, y) in [(32, 16), (32, 48)] {
        let source = &preview[(y * 64 + x) * 4..(y * 64 + x) * 4 + 4];
        let destination = &decoded.stdout[((63 - y) * 64 + x) * 4..((63 - y) * 64 + x) * 4 + 4];
        assert!(
            source
                .iter()
                .zip(destination)
                .all(|(a, b)| a.abs_diff(*b) <= 5),
            "Preview {source:?} differs from MP4 {destination:?}."
        );
    }
}
