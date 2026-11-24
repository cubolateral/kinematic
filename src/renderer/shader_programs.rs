use std::{cell::RefCell, rc::Rc};

use glow::HasContext;

use crate::core::{TrackValue, shader_uniforms::ShaderUniforms};

pub(super) const IMAGE_VERTEX: &str = "#version 330 core
out vec2 k_uv;
void main() {
    vec2 p = vec2((gl_VertexID << 1) & 2, gl_VertexID & 2);
    k_uv = p;
    gl_Position = vec4(p * 2.0 - 1.0, 0.0, 1.0);
}
";

type ContextProvider = dyn Fn() -> Result<Rc<glow::Context>, String>;

thread_local! {
    static CONTEXT: RefCell<Option<Rc<ContextProvider>>> = RefCell::new(None);
}

pub(crate) fn with_context<T>(
    provider: impl Fn() -> Result<Rc<glow::Context>, String> + 'static,
    run: impl FnOnce() -> T,
) -> T {
    struct Restore(Option<Rc<ContextProvider>>);

    impl Drop for Restore {
        fn drop(&mut self) {
            CONTEXT.with(|context| context.replace(self.0.take()));
        }
    }

    let _restore = Restore(CONTEXT.with(|context| context.replace(Some(Rc::new(provider)))));

    run()
}

pub(crate) fn reflect(
    vertex: Option<&str>,
    fragment: &str,
    image: bool,
) -> Result<Option<ShaderUniforms>, String> {
    let Some(provider) = CONTEXT.with(|context| context.borrow().clone()) else {
        if cfg!(test) {
            return Ok(None);
        }

        return Err("Shader reflection needs an OpenGL context.".into());
    };
    let gl = provider()?;
    let program = if image {
        image_program(&gl, fragment)?
    } else {
        let vertex = vertex
            .map(str::to_owned)
            .unwrap_or_else(crate::core::components::mesh_reflection_vertex);

        link(
            &gl,
            &format!("#version 330 core\n{vertex}"),
            &format!("#version 330 core\n{fragment}"),
        )?
    };

    let uniforms = unsafe {
        let uniforms = (0..gl.get_active_uniforms(program))
            .filter_map(|index| {
                let uniform = gl.get_active_uniform(program, index)?;
                let name = &uniform.name;
                let base = name.split(['[', '.']).next().unwrap_or(name);

                if base.starts_with("gl_")
                    || !(if image {
                        crate::core::objects::valid_binding_name(base)
                    } else {
                        crate::core::objects::valid_mesh_uniform_name(base)
                    })
                {
                    return None;
                }

                let location = gl.get_uniform_location(program, name)?;
                let default = if uniform.size == 1 && base == name {
                    read_value(&gl, program, &location, uniform.utype)
                } else {
                    None
                };

                Some((uniform.name, default))
            })
            .collect();

        gl.delete_program(program);

        uniforms
    };

    Ok(Some(uniforms))
}

fn read_value(
    gl: &glow::Context,
    program: glow::NativeProgram,
    location: &glow::NativeUniformLocation,
    kind: u32,
) -> Option<TrackValue> {
    unsafe {
        Some(match kind {
            glow::FLOAT | glow::FLOAT_VEC2 | glow::FLOAT_VEC3 | glow::FLOAT_VEC4 => {
                let mut value = [0.0; 4];

                gl.get_uniform_f32(program, location, &mut value);

                match kind {
                    glow::FLOAT => TrackValue::F32(value[0]),
                    glow::FLOAT_VEC2 => TrackValue::Vector2(glam::vec2(value[0], value[1])),
                    glow::FLOAT_VEC3 => {
                        TrackValue::Vector3(glam::vec3(value[0], value[1], value[2]))
                    }
                    _ => TrackValue::Quad(value.into()),
                }
            }
            glow::INT | glow::BOOL => {
                let mut value = [0];

                gl.get_uniform_i32(program, location, &mut value);

                if kind == glow::BOOL {
                    TrackValue::Bool(value[0] != 0)
                } else {
                    TrackValue::I32(value[0])
                }
            }
            glow::UNSIGNED_INT => {
                let mut value = [0];

                gl.get_uniform_u32(program, location, &mut value);

                TrackValue::U32(value[0])
            }
            _ => return None,
        })
    }
}

pub(super) fn image_program(
    gl: &glow::Context,
    fragment: &str,
) -> Result<glow::NativeProgram, String> {
    if fragment.lines().next().map(str::trim) != Some("#version 330 core") {
        return Err("Image shader must start with `#version 330 core`.".into());
    }

    link(gl, IMAGE_VERTEX, fragment)
}

fn link(gl: &glow::Context, vertex: &str, fragment: &str) -> Result<glow::NativeProgram, String> {
    unsafe {
        let program = gl.create_program()?;
        let result = (|| {
            for (kind, source) in [
                (glow::VERTEX_SHADER, vertex),
                (glow::FRAGMENT_SHADER, fragment),
            ] {
                let shader = gl.create_shader(kind)?;

                gl.shader_source(shader, source);
                gl.compile_shader(shader);

                if !gl.get_shader_compile_status(shader) {
                    let diagnostic = gl.get_shader_info_log(shader);

                    gl.delete_shader(shader);

                    return Err(format!("Shader compilation failed:\n{diagnostic}"));
                }

                gl.attach_shader(program, shader);
                gl.delete_shader(shader);
            }

            gl.link_program(program);

            if !gl.get_program_link_status(program) {
                return Err(format!(
                    "Shader link failed:\n{}",
                    gl.get_program_info_log(program)
                ));
            }

            Ok(program)
        })();

        if result.is_err() {
            gl.delete_program(program);
        }

        result
    }
}
