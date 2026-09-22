use std::{collections::HashMap, rc::Rc};

use glow::HasContext;

use crate::{
    core::{
        TrackValue,
        objects::{CanvasTexture, ImageShaderData, validate_shader_values},
    },
    renderer::target::{Target, reset_gl},
};

pub(crate) struct ImageShaders {
    programs: HashMap<u64, glow::NativeProgram>,
    vao: Option<glow::NativeVertexArray>,
    gl: Rc<glow::Context>,
}

impl ImageShaders {
    pub(crate) fn new(gl: &Rc<glow::Context>) -> Self {
        Self {
            programs: HashMap::new(),
            vao: None,
            gl: Rc::clone(gl),
        }
    }

    pub(crate) fn apply(
        &mut self,
        input: &Target,
        output: &Target,
        data: &ImageShaderData,
        time: f32,
        alpha: f32,
        texture: impl Fn(CanvasTexture) -> Option<glow::NativeTexture>,
    ) -> Result<(), String> {
        validate_shader_values(data)?;
        if data.shader.source.lines().next().map(str::trim) != Some("#version 330 core") {
            return Err("Image shader must start with `#version 330 core`.".into());
        }
        let program = self.program(data)?;
        let vao = match self.vao {
            Some(vao) => vao,
            None => {
                let vao = unsafe { self.gl.create_vertex_array()? };
                self.vao = Some(vao);
                vao
            }
        };
        let maximum = unsafe { self.gl.get_parameter_i32(glow::MAX_TEXTURE_IMAGE_UNITS) } as usize;
        if data.textures.len() + 1 > maximum {
            return Err(format!(
                "Image shader uses {} textures, but OpenGL supports {maximum}.",
                data.textures.len() + 1
            ));
        }
        for (name, expected) in [
            ("k_image", glow::SAMPLER_2D),
            ("k_resolution", glow::FLOAT_VEC2),
            ("k_time", glow::FLOAT),
            ("k_alpha", glow::FLOAT),
        ] {
            if unsafe { self.gl.get_uniform_location(program, name) }.is_some() {
                unsafe { validate_uniform_type(&self.gl, program, name, expected) }?;
            }
        }

        reset_gl(&self.gl, output.size);
        unsafe {
            self.gl
                .bind_framebuffer(glow::DRAW_FRAMEBUFFER, Some(output.framebuffer()));
            self.gl.use_program(Some(program));
            self.gl.bind_vertex_array(Some(vao));
            bind_texture(&self.gl, program, "k_image", 0, input.texture());
            uniform_2f(
                &self.gl,
                program,
                "k_resolution",
                output.size.0 as f32,
                output.size.1 as f32,
            );
            uniform_1f(&self.gl, program, "k_time", time);
            uniform_1f(&self.gl, program, "k_alpha", alpha);

            for (name, value) in &data.uniforms {
                let location = self.gl.get_uniform_location(program, name).ok_or_else(|| {
                    format!("Image shader uniform `{name}` is missing or inactive.")
                })?;
                let expected = match value {
                    TrackValue::F32(_) => glow::FLOAT,
                    TrackValue::Vector2(_) => glow::FLOAT_VEC2,
                    TrackValue::Vector3(_) => glow::FLOAT_VEC3,
                    TrackValue::Quad(_) | TrackValue::Quaternion(_) | TrackValue::Color(_) => {
                        glow::FLOAT_VEC4
                    }
                    TrackValue::I32(_) => glow::INT,
                    TrackValue::U32(_) => glow::UNSIGNED_INT,
                    TrackValue::Bool(_) => glow::BOOL,
                    TrackValue::Enum(_) | TrackValue::String(_) => unreachable!(),
                };
                validate_uniform_type(&self.gl, program, name, expected)?;
                match value {
                    TrackValue::F32(value) => self.gl.uniform_1_f32(Some(&location), *value),
                    TrackValue::Vector2(value) => {
                        self.gl.uniform_2_f32(Some(&location), value.x, value.y)
                    }
                    TrackValue::Vector3(value) => {
                        self.gl
                            .uniform_3_f32(Some(&location), value.x, value.y, value.z)
                    }
                    TrackValue::Quad(value) => {
                        let [x, y, z, w] = value.to_array();
                        self.gl.uniform_4_f32(Some(&location), x, y, z, w)
                    }
                    TrackValue::Quaternion(value) => {
                        self.gl
                            .uniform_4_f32(Some(&location), value.x, value.y, value.z, value.w)
                    }
                    TrackValue::Color(value) => {
                        let [r, g, b, a] = value.rgba();
                        self.gl.uniform_4_f32(Some(&location), r, g, b, a)
                    }
                    TrackValue::I32(value) => self.gl.uniform_1_i32(Some(&location), *value),
                    TrackValue::U32(value) => self.gl.uniform_1_u32(Some(&location), *value),
                    TrackValue::Bool(value) => {
                        self.gl.uniform_1_i32(Some(&location), i32::from(*value))
                    }
                    TrackValue::Enum(_) | TrackValue::String(_) => unreachable!(),
                }
            }
            for (index, (name, canvas)) in data.textures.iter().enumerate() {
                validate_uniform_type(&self.gl, program, name, glow::SAMPLER_2D)?;
                let texture = texture(*canvas)
                    .ok_or_else(|| format!("Image shader texture `{name}` is unavailable."))?;
                bind_texture(&self.gl, program, name, index as u32 + 1, texture);
            }
            self.gl.draw_arrays(glow::TRIANGLES, 0, 3);
        }
        Ok(())
    }

    fn program(&mut self, data: &ImageShaderData) -> Result<glow::NativeProgram, String> {
        if let Some(program) = self.programs.get(&data.shader.id) {
            return Ok(*program);
        }
        let program = unsafe {
            let vertex = compile(
                &self.gl,
                glow::VERTEX_SHADER,
                "#version 330 core\nout vec2 k_uv;\nvoid main() {\n    vec2 p = vec2((gl_VertexID << 1) & 2, gl_VertexID & 2);\n    k_uv = p;\n    gl_Position = vec4(p * 2.0 - 1.0, 0.0, 1.0);\n}\n",
            )?;
            let fragment = match compile(&self.gl, glow::FRAGMENT_SHADER, &data.shader.source) {
                Ok(shader) => shader,
                Err(error) => {
                    self.gl.delete_shader(vertex);
                    return Err(error);
                }
            };
            let program = self.gl.create_program()?;
            self.gl.attach_shader(program, vertex);
            self.gl.attach_shader(program, fragment);
            self.gl.link_program(program);
            self.gl.detach_shader(program, vertex);
            self.gl.detach_shader(program, fragment);
            self.gl.delete_shader(vertex);
            self.gl.delete_shader(fragment);
            if !self.gl.get_program_link_status(program) {
                let diagnostic = self.gl.get_program_info_log(program);
                self.gl.delete_program(program);
                return Err(format!("Image shader link failed:\n{diagnostic}"));
            }
            program
        };
        self.programs.insert(data.shader.id, program);
        Ok(program)
    }
}

unsafe fn validate_uniform_type(
    gl: &glow::Context,
    program: glow::NativeProgram,
    name: &str,
    expected: u32,
) -> Result<(), String> {
    let count = unsafe { gl.get_active_uniforms(program) };
    let actual = (0..count).find_map(|index| {
        let uniform = unsafe { gl.get_active_uniform(program, index) }?;
        (uniform.name == name).then_some(uniform.utype)
    });
    match actual {
        None => Err(format!(
            "Image shader uniform `{name}` is missing or inactive."
        )),
        Some(actual) if actual != expected => Err(format!(
            "Image shader uniform `{name}` has an incompatible GLSL type."
        )),
        Some(_) => Ok(()),
    }
}

unsafe fn compile(
    gl: &glow::Context,
    kind: u32,
    source: &str,
) -> Result<glow::NativeShader, String> {
    let shader = unsafe { gl.create_shader(kind)? };
    unsafe {
        gl.shader_source(shader, source);
        gl.compile_shader(shader);
        if !gl.get_shader_compile_status(shader) {
            let diagnostic = gl.get_shader_info_log(shader);
            gl.delete_shader(shader);
            return Err(format!("Image shader compilation failed:\n{diagnostic}"));
        }
    }
    Ok(shader)
}

unsafe fn bind_texture(
    gl: &glow::Context,
    program: glow::NativeProgram,
    name: &str,
    unit: u32,
    texture: glow::NativeTexture,
) {
    unsafe {
        gl.active_texture(glow::TEXTURE0 + unit);
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));
        if let Some(location) = gl.get_uniform_location(program, name) {
            gl.uniform_1_i32(Some(&location), unit as i32);
        }
    }
}

unsafe fn uniform_1f(gl: &glow::Context, program: glow::NativeProgram, name: &str, value: f32) {
    if let Some(location) = unsafe { gl.get_uniform_location(program, name) } {
        unsafe { gl.uniform_1_f32(Some(&location), value) };
    }
}

unsafe fn uniform_2f(gl: &glow::Context, program: glow::NativeProgram, name: &str, x: f32, y: f32) {
    if let Some(location) = unsafe { gl.get_uniform_location(program, name) } {
        unsafe { gl.uniform_2_f32(Some(&location), x, y) };
    }
}

impl Drop for ImageShaders {
    fn drop(&mut self) {
        unsafe {
            for (_, program) in self.programs.drain() {
                self.gl.delete_program(program);
            }
            if let Some(vao) = self.vao.take() {
                self.gl.delete_vertex_array(vao);
            }
        }
    }
}
