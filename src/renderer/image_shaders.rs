use std::{collections::HashMap, rc::Rc};

use glow::HasContext;

use crate::{
    core::{
        TrackValue,
        objects::{CanvasTexture, ImageShaderData, validate_shader_values},
    },
    renderer::{
        shader_programs::image_program,
        target::{Target, reset_gl},
    },
};

#[derive(PartialEq, Eq, Hash)]
struct ProgramKey {
    shader: u64,
    uniforms: Vec<String>,
    textures: Vec<String>,
}

pub(crate) struct ImageShaders {
    programs: HashMap<ProgramKey, glow::NativeProgram>,
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

                if let TrackValue::Color(value) = value {
                    let [r, g, b, a] = value.rgba();

                    match active_uniform_type(&self.gl, program, name)? {
                        glow::FLOAT_VEC3 => self.gl.uniform_3_f32(Some(&location), r, g, b),
                        glow::FLOAT_VEC4 => self.gl.uniform_4_f32(Some(&location), r, g, b, a),
                        _ => {
                            return Err(format!(
                                "Image shader uniform `{name}` has an incompatible GLSL type."
                            ));
                        }
                    }

                    continue;
                }

                let expected = match value {
                    TrackValue::F32(_) => glow::FLOAT,
                    TrackValue::Vector2(_) => glow::FLOAT_VEC2,
                    TrackValue::Vector3(_) => glow::FLOAT_VEC3,
                    TrackValue::Quad(_) | TrackValue::Quaternion(_) => glow::FLOAT_VEC4,
                    TrackValue::Color(_) => unreachable!(),
                    TrackValue::I32(_) => glow::INT,
                    TrackValue::U32(_) => glow::UNSIGNED_INT,
                    TrackValue::Bool(_) => glow::BOOL,
                    TrackValue::Enum(_) | TrackValue::String(_) | TrackValue::CodeRanges(_) => {
                        unreachable!()
                    }
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
                    TrackValue::Color(_) => unreachable!(),
                    TrackValue::I32(value) => self.gl.uniform_1_i32(Some(&location), *value),
                    TrackValue::U32(value) => self.gl.uniform_1_u32(Some(&location), *value),
                    TrackValue::Bool(value) => {
                        self.gl.uniform_1_i32(Some(&location), i32::from(*value))
                    }
                    TrackValue::Enum(_) | TrackValue::String(_) | TrackValue::CodeRanges(_) => {
                        unreachable!()
                    }
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
        // Keep omitted bindings at their GLSL defaults by sharing programs only across equal binding sets.
        let key = ProgramKey {
            shader: data.shader.id,
            uniforms: data.uniforms.keys().cloned().collect(),
            textures: data.textures.keys().cloned().collect(),
        };

        if let Some(program) = self.programs.get(&key) {
            return Ok(*program);
        }

        let program = image_program(&self.gl, &data.shader.source)?;

        self.programs.insert(key, program);

        Ok(program)
    }
}

unsafe fn validate_uniform_type(
    gl: &glow::Context,
    program: glow::NativeProgram,
    name: &str,
    expected: u32,
) -> Result<(), String> {
    let actual = unsafe { active_uniform_type(gl, program, name) };

    match actual {
        Err(error) => Err(error),
        Ok(actual) if actual != expected => Err(format!(
            "Image shader uniform `{name}` has an incompatible GLSL type."
        )),
        Ok(_) => Ok(()),
    }
}

unsafe fn active_uniform_type(
    gl: &glow::Context,
    program: glow::NativeProgram,
    name: &str,
) -> Result<u32, String> {
    let count = unsafe { gl.get_active_uniforms(program) };

    (0..count)
        .find_map(|index| {
            let uniform = unsafe { gl.get_active_uniform(program, index) }?;

            (uniform.name == name).then_some(uniform.utype)
        })
        .ok_or_else(|| format!("Image shader uniform `{name}` is missing or inactive."))
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
