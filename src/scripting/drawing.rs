use super::{
    extension::{self, ScriptObject, ScriptState},
    value::ValueHandle,
};
use crate::core::{
    TrackValue,
    components::{GeometryKey, Material, RenderContext3D, Simulation},
    objects::global_matrix3d,
};
use mlua::{Lua, Result, Table, UserData, UserDataMethods, Value};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::Rc,
    sync::{
        Arc, Weak,
        atomic::{AtomicU64, Ordering},
    },
};

#[derive(Clone, Debug)]
pub(super) struct Matrix(pub glam::Mat4);
impl UserData for Matrix {
    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        m.add_meta_method(
            mlua::MetaMethod::Mul,
            |_, this, rhs: mlua::UserDataRef<Self>| Ok(Self(this.0 * rhs.0)),
        );
        m.add_method("translate", |_, this, v: mlua::UserDataRef<ValueHandle>| {
            let TrackValue::Vector3(v) = v.0 else {
                return Err(mlua::Error::runtime("Translation needs vec3."));
            };
            Ok(Self(this.0 * glam::Mat4::from_translation(v)))
        });
        m.add_method("scale", |_, this, v: mlua::UserDataRef<ValueHandle>| {
            let TrackValue::Vector3(v) = v.0 else {
                return Err(mlua::Error::runtime("Scale needs vec3."));
            };
            Ok(Self(this.0 * glam::Mat4::from_scale(v)))
        });
        m.add_method("rotate", |_, this, v: mlua::UserDataRef<ValueHandle>| {
            let TrackValue::Quaternion(v) = v.0 else {
                return Err(mlua::Error::runtime("Rotation needs a quaternion."));
            };
            Ok(Self(this.0 * glam::Mat4::from_quat(v)))
        });
    }
}
#[derive(Clone)]
pub(super) struct PathHandle(pub skia_safe::PathBuilder);
impl UserData for PathHandle {
    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        m.add_method_mut("move_to", |_, this, (x, y): (f32, f32)| {
            finite(&[x, y])?;
            this.0.move_to((x, y));
            Ok(())
        });
        m.add_method_mut("line_to", |_, this, (x, y): (f32, f32)| {
            finite(&[x, y])?;
            this.0.line_to((x, y));
            Ok(())
        });
        m.add_method_mut(
            "quad_to",
            |_, this, (x1, y1, x2, y2): (f32, f32, f32, f32)| {
                finite(&[x1, y1, x2, y2])?;
                this.0.quad_to((x1, y1), (x2, y2));
                Ok(())
            },
        );
        m.add_method_mut(
            "cubic_to",
            |_, this, (x1, y1, x2, y2, x3, y3): (f32, f32, f32, f32, f32, f32)| {
                finite(&[x1, y1, x2, y2, x3, y3])?;
                this.0.cubic_to((x1, y1), (x2, y2), (x3, y3));
                Ok(())
            },
        );
        m.add_method_mut("close", |_, this, ()| {
            this.0.close();
            Ok(())
        });
    }
}
fn finite(values: &[f32]) -> Result<()> {
    if values.iter().all(|v| v.is_finite()) {
        Ok(())
    } else {
        Err(mlua::Error::runtime("Drawing coordinates must be finite."))
    }
}
#[derive(Clone)]
pub(super) struct MeshHandle(Arc<MeshData>);
struct MeshData {
    id: u64,
    mesh: three_d::CpuMesh,
}
impl std::fmt::Debug for MeshHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Mesh").field(&self.0.id).finish()
    }
}
impl UserData for MeshHandle {}
static NEXT_MESH: AtomicU64 = AtomicU64::new(1);
thread_local! {static MESHES:RefCell<BTreeMap<String,Weak<MeshData>>>=RefCell::new(BTreeMap::new());}

pub(super) fn release_unused() {
    MESHES.with(|cache| {
        cache
            .borrow_mut()
            .retain(|_, value| value.strong_count() > 0)
    });
}

pub(super) fn register(lua: &Lua, api: &Table) -> Result<()> {
    api.set(
        "path",
        lua.create_function(|_, ()| Ok(PathHandle(skia_safe::PathBuilder::new())))?,
    )?;
    api.set(
        "mat4",
        lua.create_function(|_, values: Option<Vec<f32>>| {
            let matrix = if let Some(values) = values {
                finite(&values)?;
                let values: [f32; 16] = values.try_into().map_err(|_| {
                    mlua::Error::runtime("Matrix needs sixteen column-major values.")
                })?;
                glam::Mat4::from_cols_array(&values)
            } else {
                glam::Mat4::IDENTITY
            };
            Ok(Matrix(matrix))
        })?,
    )?;
    api.set(
        "mesh",
        lua.create_function(|_, options: Table| {
            let vertices = options.get::<Vec<Vec<f32>>>("vertices")?;
            if vertices.is_empty() || vertices.len() > 1_000_000 {
                return Err(mlua::Error::runtime(
                    "Mesh needs between one and one million vertices.",
                ));
            }
            let vector3 = |v: Vec<f32>| -> Result<three_d::Vec3> {
                finite(&v)?;
                if v.len() != 3 {
                    return Err(mlua::Error::runtime("Mesh vertex needs three components."));
                }
                Ok(three_d::vec3(v[0], v[1], v[2]))
            };
            let positions = vertices
                .into_iter()
                .map(vector3)
                .collect::<Result<Vec<_>>>()?;
            let indices = options.get::<Vec<u32>>("indices")?;
            if !indices.len().is_multiple_of(3)
                || indices.len() > 3_000_000
                || indices.iter().any(|i| *i as usize >= positions.len())
            {
                return Err(mlua::Error::runtime(
                    "Mesh indices must be zero-based valid triangles.",
                ));
            }
            let normals = options
                .get::<Option<Vec<Vec<f32>>>>("normals")?
                .map(|vs| vs.into_iter().map(vector3).collect::<Result<Vec<_>>>())
                .transpose()?;
            let uvs = options
                .get::<Option<Vec<Vec<f32>>>>("uvs")?
                .map(|vs| {
                    vs.into_iter()
                        .map(|v| {
                            finite(&v)?;
                            if v.len() != 2 {
                                return Err(mlua::Error::runtime("UV needs two components."));
                            }
                            Ok(three_d::vec2(v[0], v[1]))
                        })
                        .collect::<Result<Vec<_>>>()
                })
                .transpose()?;
            let colors = options
                .get::<Option<Vec<mlua::UserDataRef<ValueHandle>>>>("colors")?
                .map(|vs| {
                    vs.into_iter()
                        .map(|v| {
                            let TrackValue::Color(c) = v.0 else {
                                return Err(mlua::Error::runtime(
                                    "Vertex color needs a color value.",
                                ));
                            };
                            let c = c.rgba().map(|x| (x.clamp(0.0, 1.0) * 255.0).round() as u8);
                            Ok(three_d::Srgba::new(c[0], c[1], c[2], c[3]))
                        })
                        .collect::<Result<Vec<_>>>()
                })
                .transpose()?;
            if normals.as_ref().is_some_and(|v| v.len() != positions.len())
                || uvs.as_ref().is_some_and(|v| v.len() != positions.len())
                || colors.as_ref().is_some_and(|v| v.len() != positions.len())
            {
                return Err(mlua::Error::runtime(
                    "Mesh attributes must match the vertex count.",
                ));
            }
            let mut mesh = three_d::CpuMesh {
                positions: three_d::Positions::F32(positions),
                indices: three_d::Indices::U32(indices),
                normals,
                uvs,
                colors,
                ..Default::default()
            };
            if mesh.normals.is_none() {
                mesh.compute_normals();
            }
            let key = format!("{mesh:?}");
            Ok(MESHES.with(|cache| {
                let mut cache = cache.borrow_mut();
                cache.retain(|_, v| v.strong_count() > 0);
                if let Some(existing) = cache.get(&key).and_then(Weak::upgrade) {
                    return MeshHandle(existing);
                };
                let data = Arc::new(MeshData {
                    id: NEXT_MESH.fetch_add(1, Ordering::Relaxed),
                    mesh,
                });
                cache.insert(key, Arc::downgrade(&data));
                MeshHandle(data)
            }))
        })?,
    )?;
    Ok(())
}

enum Command2D {
    Save,
    Restore,
    Layer(f32),
    Translate(f32, f32),
    Scale(f32, f32),
    Rotate(f32),
    Clip(skia_safe::Path),
    Path(skia_safe::Path, skia_safe::Paint),
    Text(String, f32, f32, skia_safe::Font, skia_safe::Paint),
    Image(skia_safe::Image, skia_safe::Rect),
}
#[derive(Clone)]
struct Context2D {
    active: Rc<Cell<bool>>,
    commands: Rc<RefCell<Vec<Command2D>>>,
    depth: Rc<Cell<u32>>,
}
impl Context2D {
    fn push(&self, command: Command2D) -> Result<()> {
        if !self.active.get() {
            return Err(mlua::Error::runtime("Drawing context has expired."));
        }
        let mut commands = self.commands.borrow_mut();
        if commands.len() >= 100_000 {
            return Err(mlua::Error::runtime("Drawing exceeds 100000 commands."));
        }
        commands.push(command);
        Ok(())
    }
}
fn paint(value: Value, stroke: Option<f32>) -> Result<skia_safe::Paint> {
    let mut paint = skia_safe::Paint::default();
    paint.set_anti_alias(true);
    match value {
        Value::UserData(value) => {
            let value = value.borrow::<ValueHandle>()?;
            let TrackValue::Color(color) = value.0 else {
                return Err(mlua::Error::runtime("Paint needs a color or gradient."));
            };
            let [r, g, b, a] = color.rgba();
            paint.set_color4f(skia_safe::Color4f::new(r, g, b, a), None);
        }
        Value::Table(gradient) => {
            let start = gradient.get::<Vec<f32>>("from")?;
            let end = gradient.get::<Vec<f32>>("to")?;
            if start.len() != 2 || end.len() != 2 {
                return Err(mlua::Error::runtime(
                    "Gradient endpoints need two coordinates.",
                ));
            }
            finite(&start)?;
            finite(&end)?;
            let colors = gradient
                .get::<Vec<mlua::UserDataRef<ValueHandle>>>("colors")?
                .iter()
                .map(|value| {
                    let TrackValue::Color(color) = value.0 else {
                        return Err(mlua::Error::runtime("Gradient needs colors."));
                    };
                    let [r, g, b, a] = color.rgba();
                    Ok(skia_safe::Color4f::new(r, g, b, a))
                })
                .collect::<Result<Vec<_>>>()?;
            if colors.len() < 2 {
                return Err(mlua::Error::runtime("Gradient needs at least two colors."));
            }
            let shader = skia_safe::gradient::shaders::linear_gradient(
                ((start[0], start[1]), (end[0], end[1])),
                &skia_safe::gradient::Gradient::new(
                    skia_safe::gradient::Colors::new_evenly_spaced(
                        &colors,
                        skia_safe::TileMode::Clamp,
                        None,
                    ),
                    skia_safe::gradient::Interpolation::default(),
                ),
                None,
            )
            .ok_or_else(|| mlua::Error::runtime("Invalid gradient."))?;
            paint.set_shader(shader);
        }
        _ => return Err(mlua::Error::runtime("Paint needs a color or gradient.")),
    }
    if let Some(width) = stroke {
        finite(&[width])?;
        if width < 0.0 {
            return Err(mlua::Error::runtime("Stroke width must be nonnegative."));
        }
        paint
            .set_style(skia_safe::paint::Style::Stroke)
            .set_stroke_width(width);
    }
    Ok(paint)
}
impl UserData for Context2D {
    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        m.add_method("save", |_, this, ()| {
            this.push(Command2D::Save)?;
            this.depth.set(this.depth.get() + 1);
            Ok(())
        });
        m.add_method("restore", |_, this, ()| {
            if this.depth.get() == 0 {
                return Err(mlua::Error::runtime("Unbalanced drawing restore."));
            }
            this.push(Command2D::Restore)?;
            this.depth.set(this.depth.get() - 1);
            Ok(())
        });
        m.add_method("layer", |_, this, alpha: f32| {
            finite(&[alpha])?;
            if !(0.0..=1.0).contains(&alpha) {
                return Err(mlua::Error::runtime(
                    "Layer opacity must be between zero and one.",
                ));
            }
            this.push(Command2D::Layer(alpha))?;
            this.depth.set(this.depth.get() + 1);
            Ok(())
        });
        m.add_method("translate", |_, this, (x, y): (f32, f32)| {
            finite(&[x, y])?;
            this.push(Command2D::Translate(x, y))
        });
        m.add_method("scale", |_, this, (x, y): (f32, f32)| {
            finite(&[x, y])?;
            this.push(Command2D::Scale(x, y))
        });
        m.add_method("rotate", |_, this, r: f32| {
            finite(&[r])?;
            this.push(Command2D::Rotate(r))
        });
        m.add_method("clip", |_, this, path: mlua::UserDataRef<PathHandle>| {
            this.push(Command2D::Clip(path.0.snapshot()))
        });
        m.add_method(
            "fill",
            |_, this, (path, color): (mlua::UserDataRef<PathHandle>, Value)| {
                this.push(Command2D::Path(path.0.snapshot(), paint(color, None)?))
            },
        );
        m.add_method(
            "stroke",
            |_, this, (path, color, width): (mlua::UserDataRef<PathHandle>, Value, f32)| {
                this.push(Command2D::Path(
                    path.0.snapshot(),
                    paint(color, Some(width))?,
                ))
            },
        );
        m.add_method(
            "rect",
            |_, this, (x, y, w, h, color): (f32, f32, f32, f32, Value)| {
                finite(&[x, y, w, h])?;
                this.push(Command2D::Path(
                    skia_safe::Path::rect(skia_safe::Rect::from_xywh(x, y, w, h), None),
                    paint(color, None)?,
                ))
            },
        );
        m.add_method(
            "circle",
            |_, this, (x, y, r, color): (f32, f32, f32, Value)| {
                finite(&[x, y, r])?;
                if r < 0.0 {
                    return Err(mlua::Error::runtime("Circle radius must be nonnegative."));
                }
                this.push(Command2D::Path(
                    skia_safe::Path::circle((x, y), r, None),
                    paint(color, None)?,
                ))
            },
        );
        m.add_method(
            "image",
            |_,
             this,
             (image, x, y, width, height): (
                mlua::UserDataRef<super::resources::ImageHandle>,
                f32,
                f32,
                f32,
                f32,
            )| {
                finite(&[x, y, width, height])?;
                if width < 0.0 || height < 0.0 {
                    return Err(mlua::Error::runtime("Image size must be nonnegative."));
                }
                this.push(Command2D::Image(
                    super::resources::image(&image).clone(),
                    skia_safe::Rect::from_xywh(x, y, width, height),
                ))
            },
        );
        m.add_method(
            "text",
            |_,
             this,
             (text, x, y, size, color, font): (
                String,
                f32,
                f32,
                f32,
                Value,
                Option<mlua::UserDataRef<super::resources::FontHandle>>,
            )| {
                finite(&[x, y, size])?;
                if !(0.0..=4096.0).contains(&size) || text.len() > 100_000 {
                    return Err(mlua::Error::runtime("Text exceeds drawing limits."));
                }
                let font = if let Some(font) = font {
                    font.0.skia_font(size)
                } else {
                    skia_safe::Font::new(
                        skia_safe::FontMgr::default()
                            .legacy_make_typeface(None, skia_safe::FontStyle::normal())
                            .ok_or_else(|| mlua::Error::runtime("No drawing font is available."))?,
                        size,
                    )
                };
                this.push(Command2D::Text(text, x, y, font, paint(color, None)?))
            },
        );
    }
}
fn arguments(
    runtime: &super::runtime::Runtime,
    world: &hecs::World,
    entity: hecs::Entity,
) -> Result<(Table, Value)> {
    let props = runtime.lua.create_table()?;
    for target in super::native::properties(world, entity) {
        props.set(
            target.name(),
            super::value::to_lua(&runtime.lua, target.get(world, entity))?,
        )?;
    }
    let state = world
        .get::<&Simulation>(entity)
        .ok()
        .map(|s| s.read_state::<ScriptState, _>(|s| s.data.clone()))
        .map(|s| s.value(&runtime.lua))
        .transpose()?
        .unwrap_or(Value::Nil);
    Ok((props, state))
}
pub(super) fn bounds_2d(world: &hecs::World, entity: hecs::Entity) -> skia_safe::Rect {
    let result = (|| {
        let id = world
            .get::<&ScriptObject>(entity)
            .map_err(mlua::Error::external)?
            .runtime;
        let runtime = extension::runtime(id)?;
        let callback = runtime
            .callbacks
            .borrow()
            .get(&entity)
            .and_then(|c| c.bounds.clone());
        let Some(callback) = callback else {
            return Ok(skia_safe::Rect::new_empty());
        };
        let (props, state) = arguments(&runtime, world, entity)?;
        let values = runtime.draw_call(|| callback.call::<Vec<f32>>((props, state)))?;
        if values.len() != 4 {
            return Err(mlua::Error::runtime(
                "2D bounds must return {x,y,width,height}.",
            ));
        }
        finite(&values)?;
        if values[2] < 0.0 || values[3] < 0.0 {
            return Err(mlua::Error::runtime("Bounds size cannot be negative."));
        }
        Ok(skia_safe::Rect::from_xywh(
            values[0], values[1], values[2], values[3],
        ))
    })();
    match result {
        Ok(bounds) => bounds,
        Err(error) => {
            report(world, entity, "Bounds", error);
            skia_safe::Rect::new_empty()
        }
    }
}
pub(super) fn bounds_3d(world: &hecs::World, entity: hecs::Entity) -> (glam::Vec3, glam::Vec3) {
    let result = (|| {
        let id = world
            .get::<&ScriptObject>(entity)
            .map_err(mlua::Error::external)?
            .runtime;
        let runtime = extension::runtime(id)?;
        let callback = runtime
            .callbacks
            .borrow()
            .get(&entity)
            .and_then(|c| c.bounds.clone())
            .ok_or_else(|| mlua::Error::runtime("3D drawing requires bounds."))?;
        let (props, state) = arguments(&runtime, world, entity)?;
        let values = runtime.draw_call(|| callback.call::<Vec<f32>>((props, state)))?;
        finite(&values)?;
        let (min, max) = match values.as_slice() {
            [w, h, d] => {
                let half = glam::vec3(*w, *h, *d) * 0.5;
                (-half, half)
            }
            [x, y, z, w, h, d] => {
                let min = glam::vec3(*x, *y, *z);
                (min, min + glam::vec3(*w, *h, *d))
            }
            _ => {
                return Err(mlua::Error::runtime(
                    "3D bounds require {width,height,depth} or {x,y,z,width,height,depth}.",
                ));
            }
        };
        if (max - min).min_element() < 0.0 {
            return Err(mlua::Error::runtime(
                "Bounds dimensions cannot be negative.",
            ));
        }
        Ok((min, max))
    })();
    match result {
        Ok(bounds) => bounds,
        Err(error) => {
            report(world, entity, "Bounds", error);
            (glam::Vec3::ZERO, glam::Vec3::ZERO)
        }
    }
}
fn report(world: &hecs::World, entity: hecs::Entity, phase: &str, error: mlua::Error) {
    if let Ok(object) = world.get::<&ScriptObject>(entity) {
        if let Ok(runtime) = extension::runtime(object.runtime) {
            runtime
                .errors
                .borrow_mut()
                .push(runtime.diagnostic(phase, entity, error));
        }
    }
}
pub(super) fn draw_2d(
    world: &hecs::World,
    entity: hecs::Entity,
    canvas: &skia_safe::Canvas,
    _opacity: f32,
) {
    let result = (|| {
        let id = world
            .get::<&ScriptObject>(entity)
            .map_err(mlua::Error::external)?
            .runtime;
        let runtime = extension::runtime(id)?;
        let callback = runtime
            .callbacks
            .borrow()
            .get(&entity)
            .and_then(|c| c.draw.clone());
        let Some(callback) = callback else {
            return Ok(());
        };
        let context = Context2D {
            active: Rc::new(Cell::new(true)),
            commands: Rc::new(RefCell::new(Vec::new())),
            depth: Rc::new(Cell::new(0)),
        };
        let (props, state) = arguments(&runtime, world, entity)?;
        let result = runtime.draw_call(|| callback.call::<()>((context.clone(), props, state)));
        context.active.set(false);
        result?;
        if context.depth.get() != 0 {
            return Err(mlua::Error::runtime("Unbalanced drawing save/layer."));
        }
        let save = canvas.save();
        for command in context.commands.borrow().iter() {
            match command {
                Command2D::Save => {
                    canvas.save();
                }
                Command2D::Restore => {
                    canvas.restore();
                }
                Command2D::Layer(alpha) => {
                    let mut paint = skia_safe::Paint::default();
                    paint.set_alpha_f(*alpha);
                    canvas.save_layer(&skia_safe::canvas::SaveLayerRec::default().paint(&paint));
                }
                Command2D::Translate(x, y) => {
                    canvas.translate((*x, *y));
                }
                Command2D::Scale(x, y) => {
                    canvas.scale((*x, *y));
                }
                Command2D::Rotate(r) => {
                    canvas.rotate(r.to_degrees(), None);
                }
                Command2D::Clip(path) => {
                    canvas.clip_path(path, None, true);
                }
                Command2D::Path(path, paint) => {
                    canvas.draw_path(path, paint);
                }
                Command2D::Text(text, x, y, font, paint) => {
                    canvas.draw_str(text, (*x, *y), font, paint);
                }
                Command2D::Image(image, rect) => {
                    canvas.draw_image_rect(image, None, *rect, &skia_safe::Paint::default());
                }
            }
        }
        canvas.restore_to_count(save);
        Ok(())
    })();
    if let Err(error) = result {
        report(world, entity, "Draw", error);
    }
}
#[derive(Clone)]
struct Context3D {
    active: Rc<Cell<bool>>,
    commands: Rc<RefCell<Vec<(MeshHandle, glam::Mat4)>>>,
}
impl UserData for Context3D {
    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        m.add_method(
            "mesh",
            |_,
             this,
             (mesh, matrix): (
                mlua::UserDataRef<MeshHandle>,
                Option<mlua::UserDataRef<Matrix>>,
            )| {
                if !this.active.get() {
                    return Err(mlua::Error::runtime("Drawing context has expired."));
                }
                if this.commands.borrow().len() >= 100_000 {
                    return Err(mlua::Error::runtime(
                        "Mesh submissions exceed the frame limit.",
                    ));
                }
                this.commands
                    .borrow_mut()
                    .push((mesh.clone(), matrix.map_or(glam::Mat4::IDENTITY, |m| m.0)));
                Ok(())
            },
        );
        m.add_method(
            "instances",
            |_,
             this,
             (mesh, matrices): (
                mlua::UserDataRef<MeshHandle>,
                Vec<mlua::UserDataRef<Matrix>>,
            )| {
                if !this.active.get() || matrices.len() + this.commands.borrow().len() > 100_000 {
                    return Err(mlua::Error::runtime(
                        "Invalid drawing context or instance count.",
                    ));
                }
                this.commands
                    .borrow_mut()
                    .extend(matrices.into_iter().map(|m| (mesh.clone(), m.0)));
                Ok(())
            },
        );
    }
}
pub(super) fn draw_3d(
    world: &hecs::World,
    entity: hecs::Entity,
    render: &mut RenderContext3D<'_>,
) -> std::result::Result<(), String> {
    let result = (|| {
        let id = world
            .get::<&ScriptObject>(entity)
            .map_err(mlua::Error::external)?
            .runtime;
        let runtime = extension::runtime(id)?;
        let callback = runtime
            .callbacks
            .borrow()
            .get(&entity)
            .and_then(|c| c.draw.clone());
        let Some(callback) = callback else {
            return Ok(());
        };
        let context = Context3D {
            active: Rc::new(Cell::new(true)),
            commands: Rc::new(RefCell::new(Vec::new())),
        };
        let (props, state) = arguments(&runtime, world, entity)?;
        let result = runtime.draw_call(|| callback.call::<()>((context.clone(), props, state)));
        context.active.set(false);
        result?;
        let material = world
            .get::<&Material>(entity)
            .map_err(mlua::Error::external)?;
        let global = global_matrix3d(world, entity);
        for (mesh, matrix) in context.commands.borrow().iter() {
            render
                .render_material(
                    GeometryKey::new::<MeshData>(mesh.0.id),
                    || mesh.0.mesh.clone(),
                    global * *matrix,
                    &material,
                )
                .map_err(mlua::Error::runtime)?;
        }
        Ok(())
    })();
    result.map_err(|error: mlua::Error| {
        world
            .get::<&ScriptObject>(entity)
            .ok()
            .and_then(|object| extension::runtime(object.runtime).ok())
            .map_or_else(
                || format!("Draw {entity:?}: {error}"),
                |runtime| runtime.diagnostic("Draw", entity, &error),
            )
    })
}
