use super::runtime::{ObjectHandle, Runtime};
use crate::core::{TrackValue, objects::*};
use mlua::{Lua, Result, Table, UserData, Value};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    path::Path,
    rc::Rc,
    sync::{Arc, Weak},
};
#[derive(Clone)]
pub(super) struct ImageHandle(Arc<ImageAsset>);
struct ImageAsset {
    source: ImageSource,
}
impl std::fmt::Debug for ImageHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Image")
            .field(&self.0.source.image().map(|i| i.unique_id()))
            .finish()
    }
}
impl UserData for ImageHandle {}
#[derive(Clone)]
struct ShaderHandle(Arc<Shader>);
enum Shader {
    Image(ImageShader),
    Mesh(MeshShader),
}
impl UserData for ShaderHandle {}
#[derive(Clone, Debug)]
pub(super) struct FontHandle(pub Arc<Font>);
impl UserData for FontHandle {}
#[derive(Clone)]
pub(super) struct TextureHandle {
    pub texture: CanvasTexture,
    pub runtime: std::rc::Weak<Runtime>,
}
impl UserData for TextureHandle {}
#[derive(Clone, Debug)]
pub(super) struct ResourceStamp(pub Vec<String>);
thread_local! {
    static IMAGES:RefCell<BTreeMap<Vec<u8>,Weak<ImageAsset>>>=RefCell::new(BTreeMap::new());
    static FONTS:RefCell<BTreeMap<Vec<u8>,Weak<Font>>>=RefCell::new(BTreeMap::new());
    static SHADERS:RefCell<BTreeMap<String,Weak<Shader>>>=RefCell::new(BTreeMap::new());
}
pub(super) fn register(runtime: &Rc<Runtime>, api: &Table) -> Result<()> {
    let weak = Rc::downgrade(runtime);

    api.set(
        "load_image",
        runtime.lua.create_function(move |_, path: String| {
            let runtime = weak
                .upgrade()
                .ok_or_else(|| mlua::Error::runtime("Project has been released."))?;

            runtime.construction()?;

            let bytes = runtime.resource(Path::new(&path))?;
            let image = IMAGES.with(|images| {
                let mut images = images.borrow_mut();

                images.retain(|_, v| v.strong_count() > 0);

                if let Some(image) = images.get(&bytes).and_then(Weak::upgrade) {
                    return Ok(ImageHandle(image));
                }

                let source =
                    ImageSource::decode(Path::new(&path), &bytes).map_err(mlua::Error::runtime)?;
                let asset = Arc::new(ImageAsset { source });

                images.insert(bytes, Arc::downgrade(&asset));

                Ok::<_, mlua::Error>(ImageHandle(asset))
            })?;

            runtime.assets.borrow_mut().push(Box::new(image.clone()));

            Ok(image)
        })?,
    )?;

    let weak = Rc::downgrade(runtime);

    api.set(
        "load_font",
        runtime.lua.create_function(move |_, path: String| {
            let runtime = weak
                .upgrade()
                .ok_or_else(|| mlua::Error::runtime("Project has been released."))?;

            runtime.construction()?;

            let bytes = runtime.resource(Path::new(&path))?;
            let full = runtime
                .context
                .resolve(&path)
                .map_err(mlua::Error::external)?;
            let font = FONTS.with(|cache| {
                let mut cache = cache.borrow_mut();

                cache.retain(|_, v| v.strong_count() > 0);

                if let Some(font) = cache.get(&bytes).and_then(Weak::upgrade) {
                    return Ok::<_, mlua::Error>(FontHandle(font));
                }

                let font =
                    Arc::new(Font::from_data(full, bytes.clone()).map_err(mlua::Error::runtime)?);

                cache.insert(bytes, Arc::downgrade(&font));

                Ok(FontHandle(font))
            })?;

            runtime.assets.borrow_mut().push(Box::new(font.clone()));

            Ok(font)
        })?,
    )?;

    for image in [true, false] {
        let weak = Rc::downgrade(runtime);

        api.set(
            if image { "image_shader" } else { "mesh_shader" },
            runtime.lua.create_function(move |_, options: Table| {
                let runtime = weak
                    .upgrade()
                    .ok_or_else(|| mlua::Error::runtime("Project has been released."))?;

                runtime.construction()?;

                let source = |name: &str, file: &str| -> Result<Option<String>> {
                    if let Some(path) = options.get::<Option<String>>(file)? {
                        return String::from_utf8(runtime.resource(Path::new(&path))?)
                            .map(Some)
                            .map_err(mlua::Error::external);
                    }

                    options.get(name)
                };
                let fragment = source(
                    if image { "source" } else { "fragment" },
                    if image { "path" } else { "fragment_path" },
                )?
                .ok_or_else(|| mlua::Error::runtime("Shader source is required."))?;
                let vertex = source("vertex", "vertex_path")?;
                let key = format!("{image}:{vertex:?}:{fragment}");
                let shader = SHADERS.with(|cache| -> Result<ShaderHandle> {
                    let mut cache = cache.borrow_mut();

                    cache.retain(|_, v| v.strong_count() > 0);

                    if let Some(shader) = cache.get(&key).and_then(Weak::upgrade) {
                        return Ok(ShaderHandle(shader));
                    }

                    let definitions = crate::renderer::shader_programs::reflect(
                        vertex.as_deref(),
                        &fragment,
                        image,
                    )
                    .map_err(mlua::Error::runtime)?
                    .map(Arc::new);
                    let shader = Arc::new(if image {
                        let mut shader = ImageShader::new(fragment);
                        shader.uniforms = definitions;

                        Shader::Image(shader)
                    } else {
                        let mut shader = if let Some(vertex) = vertex {
                            MeshShader::program(vertex, fragment)
                        } else {
                            MeshShader::fragment(fragment)
                        };
                        shader.uniforms = definitions;

                        Shader::Mesh(shader)
                    });

                    cache.insert(key, Arc::downgrade(&shader));

                    Ok(ShaderHandle(shader))
                })?;

                runtime.assets.borrow_mut().push(Box::new(shader.clone()));

                Ok(shader)
            })?,
        )?;
    }

    Ok(())
}
pub(super) fn reserved(name: &str) -> bool {
    matches!(
        name,
        "source"
            | "shader"
            | "uniforms"
            | "textures"
            | "shader_padding"
            | "resolution"
            | "font"
            | "segments"
            | "pixels_per_unit"
    )
}
pub(super) fn configure(
    lua: &Lua,
    runtime: &Rc<Runtime>,
    object: &ObjectHandle,
    options: &Table,
) -> Result<()> {
    let entity = object.entity;
    let world = runtime.world.borrow();
    let mut world = world.borrow_mut();
    let mut stamp = Vec::new();

    if let Some(size) = options.get::<Option<[u32; 2]>>("resolution")? {
        let size = (size[0], size[1]);
        let mut settings = world
            .get::<&mut CanvasSettings>(entity)
            .map_err(mlua::Error::external)?;

        if size.0 > 8192 || size.1 > 8192 {
            return Err(mlua::Error::runtime(
                "Canvas dimensions cannot exceed 8192 pixels.",
            ));
        }

        settings.resolution = size;

        settings.validate().map_err(mlua::Error::runtime)?;

        stamp.push(format!("Resolution:{size:?}"));
    }

    if let Some(segments) = options.get::<Option<u32>>("segments")? {
        if !(3..=256).contains(&segments) {
            return Err(mlua::Error::runtime(
                "Sphere segments must be between 3 and 256.",
            ));
        }

        world
            .get::<&mut SphereShape>(entity)
            .map_err(mlua::Error::external)?
            .segments = segments;

        stamp.push(format!("Segments:{segments}"));
    }

    let pixels = options
        .get::<Option<f32>>("pixels_per_unit")?
        .unwrap_or(256.0);

    if !pixels.is_finite() || pixels <= 0.0 {
        return Err(mlua::Error::runtime(
            "Pixels per unit must be positive and finite.",
        ));
    }

    if let Some(source) = options.get::<Option<mlua::AnyUserData>>("source")? {
        if source.is::<ImageHandle>() {
            let image = source.borrow::<ImageHandle>()?;

            if world.get::<&ImageSource>(entity).is_err() {
                return Err(mlua::Error::runtime(
                    "Image source requires an image object.",
                ));
            }

            let (w, h) = image.0.source.dimensions();

            world
                .insert_one(entity, image.0.source.clone())
                .map_err(mlua::Error::external)?;

            let size = glam::vec2(w as f32, h as f32);

            if let Ok(mut shape) = world.get::<&mut PanelShape>(entity) {
                shape.size = size;
            } else if let Ok(mut shape) = world.get::<&mut PlaneShape>(entity) {
                shape.size = size / pixels;
            }

            stamp.push(format!(
                "Image:{:?}:{pixels}",
                image.0.source.image().unwrap().unique_id()
            ));
        } else {
            let texture = source.borrow::<TextureHandle>()?;

            validate_texture(&texture, runtime)?;

            let size = world
                .get::<&CanvasSettings>(texture.texture.entity)
                .map_err(mlua::Error::external)?
                .resolution();

            world
                .get::<&mut ProjectionSource>(entity)
                .map_err(mlua::Error::external)?
                .0 = Some(texture.texture);

            if let Ok(mut shape) = world.get::<&mut PanelShape>(entity) {
                shape.size = glam::vec2(size.0 as f32, size.1 as f32);
            } else if let Ok(mut shape) = world.get::<&mut PlaneShape>(entity) {
                shape.size = glam::vec2(size.0 as f32, size.1 as f32) / pixels;
            }

            stamp.push(format!("Projection:{:?}:{pixels}", texture.texture));
        }
    }

    if let Some(font) = options.get::<Option<mlua::UserDataRef<FontHandle>>>("font")? {
        if let Ok(mut text) = world.get::<&mut TextShape>(entity) {
            text.font = font.0.as_ref().clone();
        } else if let Ok(mut text) = world.get::<&mut Text3DShape>(entity) {
            text.font = font.0.as_ref().clone();
        } else if let Ok(mut code) = world.get::<&mut CodeShape>(entity) {
            code.font = font.0.as_ref().clone();
        } else {
            return Err(mlua::Error::runtime("Font requires a text object."));
        }

        stamp.push(format!("Font:{:?}", font.0));
    }

    if let Some(shader) = options.get::<Option<mlua::UserDataRef<ShaderHandle>>>("shader")? {
        let uniforms = options.get::<Option<Table>>("uniforms")?;
        let padding = options.get::<Option<f32>>("shader_padding")?.unwrap_or(0.0);

        if !padding.is_finite() || padding < 0.0 {
            return Err(mlua::Error::runtime(
                "Shader padding must be finite and nonnegative.",
            ));
        }

        let definitions = match shader.0.as_ref() {
            Shader::Image(shader) => shader.uniforms.as_deref(),
            Shader::Mesh(shader) => shader.uniforms.as_deref(),
        };
        let mut values = definitions
            .into_iter()
            .flatten()
            .filter_map(|(name, value)| Some((name.clone(), value.as_ref()?.clone())))
            .collect::<BTreeMap<_, _>>();

        if let Some(uniforms) = uniforms {
            for pair in uniforms.pairs::<String, Value>() {
                let (name, value) = pair?;
                let value = uniform_override(lua, &name, value, definitions)?;

                values.insert(name, value);
            }
        }

        match shader.0.as_ref() {
            Shader::Image(shader) => {
                let mut data = ImageShaderData::new(shader.clone());

                data.uniforms = values;
                data.padding = padding;

                if let Some(textures) = options.get::<Option<Table>>("textures")? {
                    for pair in textures.pairs::<String, mlua::UserDataRef<TextureHandle>>() {
                        let (name, texture) = pair?;

                        validate_texture(&texture, runtime)?;

                        data.textures.insert(name, texture.texture);
                    }
                }

                crate::core::objects::validate_shader_values(&data)
                    .map_err(mlua::Error::runtime)?;

                stamp.push(format!(
                    "Shader:{}:{padding}:{:?}",
                    shader.source, data.textures
                ));

                world
                    .insert_one(entity, data)
                    .map_err(mlua::Error::external)?;
            }
            Shader::Mesh(shader) => {
                let mut data = MeshShaderData::with_shader(shader.clone());

                validate_mesh_uniforms(&values).map_err(mlua::Error::runtime)?;

                data.uniforms = values;
                data.bounds_padding = Some(padding);

                stamp.push(format!(
                    "Shader:{:?}:{}:{padding}",
                    shader.vertex, shader.fragment
                ));

                world
                    .insert_one(entity, data)
                    .map_err(mlua::Error::external)?;
            }
        }
    } else if options.contains_key("uniforms")? || options.contains_key("textures")? {
        return Err(mlua::Error::runtime("Shader bindings require a shader."));
    }

    if !stamp.is_empty() {
        world
            .insert_one(entity, ResourceStamp(stamp))
            .map_err(mlua::Error::external)?;
    }

    Ok(())
}

fn uniform_override(
    lua: &Lua,
    name: &str,
    value: Value,
    definitions: Option<&crate::core::shader_uniforms::ShaderUniforms>,
) -> Result<TrackValue> {
    let value = if let Some(definitions) = definitions {
        let expected = definitions
            .get(name)
            .and_then(Option::as_ref)
            .ok_or_else(|| {
                mlua::Error::runtime(format!(
                    "Shader uniform `{name}` is missing, inactive, or unsupported."
                ))
            })?;
        let inferred = super::extension::infer(value.clone())?;

        match (&inferred, expected) {
            (TrackValue::Color(_), TrackValue::Vector3(_) | TrackValue::Quad(_))
            | (TrackValue::Quaternion(_), TrackValue::Quad(_)) => inferred,
            _ => super::value::from_lua(lua, value, expected, crate::core::TrackChoices::None)
                .map_err(|error| {
                    mlua::Error::runtime(format!("Shader uniform `{name}`: {error}"))
                })?,
        }
    } else {
        super::extension::infer(value)?
    };
    validate_shader_track_value(&value).map_err(mlua::Error::runtime)?;

    Ok(value)
}

fn validate_texture(texture: &TextureHandle, runtime: &Rc<Runtime>) -> Result<()> {
    if texture
        .runtime
        .upgrade()
        .is_none_or(|r| !Rc::ptr_eq(&r, runtime))
    {
        return Err(mlua::Error::runtime(
            "Canvas texture belongs to a different scene revision.",
        ));
    }

    Ok(())
}
pub(super) fn image(handle: &ImageHandle) -> &skia_safe::Image {
    handle.0.source.image().unwrap()
}

pub(super) fn release_unused() {
    IMAGES.with(|cache| cache.borrow_mut().retain(|_, v| v.strong_count() > 0));

    FONTS.with(|cache| cache.borrow_mut().retain(|_, v| v.strong_count() > 0));

    SHADERS.with(|cache| cache.borrow_mut().retain(|_, v| v.strong_count() > 0));
}
