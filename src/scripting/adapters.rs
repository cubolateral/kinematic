use super::{
    extension::{Script2D, Script2DHandler, Script3D, Script3DHandler},
    runtime::{ObjectHandle, TaskHandle, duration, engine, parse_easing},
    value::ValueHandle,
};
use crate::core::{TrackValue, components::*, objects::*};
use mlua::{Result, UserData, UserDataMethods};
use std::rc::Weak;

struct SnapshotHandle {
    value: Snapshot<Script2DHandler>,
    object: ObjectHandle,
}
impl UserData for SnapshotHandle {}
fn proxy(object: &ObjectHandle) -> Result<Script2DHandler> {
    let runtime = object.runtime()?;
    Ok(Script2D::handler(
        runtime.world.borrow().clone(),
        object.entity,
        runtime.animator.borrow().clone(),
    ))
}
fn two_d(object: &ObjectHandle) -> Result<Script2DHandler> {
    let runtime = object.runtime()?;
    let world = runtime.world.borrow();
    let world = world.borrow();
    if world.get::<&Transform2D>(object.entity).is_err()
        || world.get::<&Draw2D>(object.entity).is_err()
    {
        return Err(mlua::Error::runtime("This operation requires a 2D object."));
    }
    drop(world);
    proxy(object)
}
fn three_d(object: &ObjectHandle) -> Result<Script3DHandler> {
    let runtime = object.runtime()?;
    if !is_3d(object)? {
        return Err(mlua::Error::runtime("This operation requires a 3D object."));
    }
    Ok(Script3D::handler(
        runtime.world.borrow().clone(),
        object.entity,
        runtime.animator.borrow().clone(),
    ))
}
fn is_3d(object: &ObjectHandle) -> Result<bool> {
    let runtime = object.runtime()?;
    Ok(runtime
        .world
        .borrow()
        .borrow()
        .get::<&Transform3D>(object.entity)
        .is_ok())
}

pub(super) const ANCHORS: &[(&str, f32, f32, f32)] = &[
    ("middle", 0.0, 0.0, 0.0),
    ("left", -1.0, 0.0, 0.0),
    ("right", 1.0, 0.0, 0.0),
    ("top", 0.0, 1.0, 0.0),
    ("bottom", 0.0, -1.0, 0.0),
    ("top_left", -1.0, 1.0, 0.0),
    ("top_right", 1.0, 1.0, 0.0),
    ("bottom_left", -1.0, -1.0, 0.0),
    ("bottom_right", 1.0, -1.0, 0.0),
    ("front", 0.0, 0.0, 1.0),
    ("back", 0.0, 0.0, -1.0),
    ("left_front", -1.0, 0.0, 1.0),
    ("right_front", 1.0, 0.0, 1.0),
    ("top_front", 0.0, 1.0, 1.0),
    ("bottom_front", 0.0, -1.0, 1.0),
    ("top_left_front", -1.0, 1.0, 1.0),
    ("top_right_front", 1.0, 1.0, 1.0),
    ("bottom_left_front", -1.0, -1.0, 1.0),
    ("bottom_right_front", 1.0, -1.0, 1.0),
    ("left_back", -1.0, 0.0, -1.0),
    ("right_back", 1.0, 0.0, -1.0),
    ("top_back", 0.0, 1.0, -1.0),
    ("bottom_back", 0.0, -1.0, -1.0),
    ("top_left_back", -1.0, 1.0, -1.0),
    ("top_right_back", 1.0, 1.0, -1.0),
    ("bottom_left_back", -1.0, -1.0, -1.0),
    ("bottom_right_back", 1.0, -1.0, -1.0),
];
pub(super) fn object_methods<M: UserDataMethods<ObjectHandle>>(m: &mut M) {
    m.add_method("snapshot", |_, this, ()| {
        Ok(SnapshotHandle {
            value: proxy(this)?.snapshot(),
            object: this.clone(),
        })
    });
    m.add_method("save", |_, this, ()| {
        this.runtime()?.construction()?;
        let handler = proxy(this)?;
        engine(|| handler.save())?;
        Ok(())
    });
    m.add_method(
        "restore",
        |_,
         this,
         (snapshot, seconds, ease): (
            Option<mlua::UserDataRef<SnapshotHandle>>,
            Option<f32>,
            Option<String>,
        )| {
            this.runtime()?.construction()?;
            let handler = proxy(this)?;
            let seconds = duration(seconds.unwrap_or(1.0))?;
            let ease = parse_easing(ease)?;
            if let Some(snapshot) = snapshot {
                if !Weak::ptr_eq(&this.runtime, &snapshot.object.runtime)
                    || this.entity != snapshot.object.entity
                {
                    return Err(mlua::Error::runtime(
                        "Snapshot belongs to a different object.",
                    ));
                }
                engine(|| {
                    TaskHandle(
                        handler
                            .restore_snapshot(&snapshot.value)
                            .duration(seconds)
                            .easing(ease)
                            .task(),
                    )
                })
            } else {
                engine(|| TaskHandle(handler.restore().duration(seconds).easing(ease).task()))
            }
        },
    );
    m.add_method(
        "reset",
        |_, this, (seconds, ease): (Option<f32>, Option<String>)| {
            this.runtime()?.construction()?;
            let handler = proxy(this)?;
            let time = duration(seconds.unwrap_or(1.0))?;
            let ease = parse_easing(ease)?;
            engine(|| TaskHandle(handler.reset().duration(time).easing(ease).task()))
        },
    );
    m.add_method("children", |_, this, ()| {
        let runtime = this.runtime()?;
        let children = children(&runtime.world.borrow().borrow(), this.entity);
        Ok(children
            .into_iter()
            .map(|entity| ObjectHandle {
                runtime: this.runtime.clone(),
                entity,
            })
            .collect::<Vec<_>>())
    });
    m.add_method("parent", |_, this, ()| {
        let runtime = this.runtime()?;
        let entity = runtime
            .world
            .borrow()
            .borrow()
            .get::<&TreeNode>(this.entity)
            .map_err(mlua::Error::external)?
            .parent;
        Ok(entity.map(|entity| ObjectHandle {
            runtime: this.runtime.clone(),
            entity,
        }))
    });
    m.add_method("move_to", |_, this, index: usize| {
        this.runtime()?.construction()?;
        if index == 0 {
            return Err(mlua::Error::runtime("Child positions are one-based."));
        }
        let handler = proxy(this)?;
        engine(|| handler.move_to(index - 1))
    });
    for (name, direction) in [
        ("move_up", 0),
        ("move_down", 1),
        ("move_to_top", 2),
        ("move_to_bottom", 3),
    ] {
        m.add_method(name, move |_, this, ()| {
            this.runtime()?.construction()?;
            let handler = proxy(this)?;
            engine(|| match direction {
                0 => handler.move_up(),
                1 => handler.move_down(),
                2 => handler.move_to_top(),
                _ => handler.move_to_bottom(),
            })
        });
    }
    m.add_method("bounds", |_, this, ()| {
        Ok(ValueHandle(if is_3d(this)? {
            TrackValue::Vector3(three_d(this)?.box_size())
        } else {
            TrackValue::Vector2(two_d(this)?.box_size())
        }))
    });
    m.add_method("global_matrix", |_, this, ()| {
        let runtime = this.runtime()?;
        let world = runtime.world.borrow();
        let world = world.borrow();
        if world.get::<&Transform3D>(this.entity).is_ok() {
            Ok(super::drawing::Matrix(global_matrix3d(&world, this.entity)))
        } else {
            let p = object_global_position(&world, this.entity);
            let scale = object_global_scale(&world, this.entity);
            let angle = object_global_rotation(&world, this.entity);
            Ok(super::drawing::Matrix(
                glam::Mat4::from_scale_rotation_translation(
                    scale.extend(1.0),
                    glam::Quat::from_rotation_z(angle),
                    p.extend(0.0),
                ),
            ))
        }
    });
    for &(name, x, y, z) in ANCHORS {
        m.add_method(name, move |_, this, ()| {
            Ok(ValueHandle(if is_3d(this)? {
                TrackValue::Vector3(three_d(this)?.box_point(x, y, z))
            } else if z == 0.0 {
                TrackValue::Vector2(two_d(this)?.box_point(x, -y))
            } else {
                return Err(mlua::Error::runtime("Depth anchors require a 3D object."));
            }))
        });
        m.add_method(
            format!("{name}_to"),
            move |_, this, point: mlua::UserDataRef<ValueHandle>| {
                Ok(ValueHandle(match point.0 {
                    TrackValue::Vector3(point) => {
                        TrackValue::Vector3(three_d(this)?.box_point_to(point, x, y, z))
                    }
                    TrackValue::Vector2(point) if z == 0.0 => {
                        TrackValue::Vector2(two_d(this)?.box_point_to(point, x, -y))
                    }
                    _ => {
                        return Err(mlua::Error::runtime(
                            "Anchor target must match the object's dimension.",
                        ));
                    }
                }))
            },
        );
    }
    for name in ["global_position", "global_scale", "global_rotation"] {
        m.add_method(name, move |lua, this, ()| {
            let value = if is_3d(this)? {
                let handler = three_d(this)?;
                match name {
                    "global_position" => TrackValue::Vector3(handler.global_position()),
                    "global_scale" => TrackValue::Vector3(handler.global_scale()),
                    _ => TrackValue::Quaternion(handler.global_rotation()),
                }
            } else {
                let handler = two_d(this)?;
                match name {
                    "global_position" => TrackValue::Vector2(handler.global_position()),
                    "global_scale" => TrackValue::Vector2(handler.global_scale()),
                    _ => TrackValue::F32(handler.global_rotation()),
                }
            };
            super::value::to_lua(lua, value)
        });
    }
    m.add_method("global_opacity", |_, this, ()| {
        Ok(two_d(this)?.global_opacity())
    });
    m.add_method("type_name", |_, this, ()| {
        let runtime = this.runtime()?;
        Ok(runtime
            .world
            .borrow()
            .borrow()
            .get::<&Inspection>(this.entity)
            .map_err(mlua::Error::external)?
            .object_name
            .to_owned())
    });
    m.add_method("get_child", |_, this, index: usize| {
        let runtime = this.runtime()?;
        let world = runtime.world.borrow();
        let world = world.borrow();
        let entity = index
            .checked_sub(1)
            .and_then(|index| children(&world, this.entity).get(index).copied())
            .ok_or_else(|| {
                mlua::Error::runtime("Child index is out of range; positions are one-based.")
            })?;
        Ok(ObjectHandle {
            runtime: this.runtime.clone(),
            entity,
        })
    });
    m.add_method("ancestor", |_, this, type_name: String| {
        let runtime = this.runtime()?;
        let world = runtime.world.borrow();
        let world = world.borrow();
        let mut current = world
            .get::<&TreeNode>(this.entity)
            .ok()
            .and_then(|n| n.parent);
        while let Some(entity) = current {
            if world
                .get::<&Inspection>(entity)
                .is_ok_and(|i| i.object_name == type_name)
            {
                return Ok(Some(ObjectHandle {
                    runtime: this.runtime.clone(),
                    entity,
                }));
            }
            current = world.get::<&TreeNode>(entity).ok().and_then(|n| n.parent);
        }
        Ok(None)
    });
    m.add_method(
        "insert",
        |_, this, (child, index): (mlua::UserDataRef<ObjectHandle>, usize)| {
            let runtime = this.runtime()?;
            runtime.construction()?;
            child.runtime()?;
            if !Weak::ptr_eq(&this.runtime, &child.runtime) {
                return Err(mlua::Error::runtime("Objects belong to different scenes."));
            }
            let world = runtime.world.borrow();
            let len = children(&world.borrow(), this.entity).len();
            if index == 0 || index > len + 1 {
                return Err(mlua::Error::runtime(
                    "Child index is out of range; positions are one-based.",
                ));
            }
            engine(|| {
                runtime.animator.borrow().assert_finite_scope();
                attach_child(
                    &world,
                    this.entity,
                    child.entity,
                    runtime.animator.borrow().time(),
                );
                if index <= len {
                    move_child(&world, child.entity, |_, _| index - 1);
                }
            })
        },
    );
    m.add_method("remove_children", |_, this, ()| {
        let runtime = this.runtime()?;
        runtime.construction()?;
        engine(|| {
            runtime.animator.borrow().assert_finite_scope();
            let world = runtime.world.borrow();
            let world = world.borrow();
            for child in children(&world, this.entity) {
                deactivate_subtree(&world, child, runtime.animator.borrow().time());
            }
        })
    });
    m.add_method(
        "get_pixel_color",
        |_, this, point: mlua::UserDataRef<ValueHandle>| {
            let TrackValue::Vector2(point) = point.0 else {
                return Err(mlua::Error::runtime("Pixel coordinates must be vec2."));
            };
            let runtime = this.runtime()?;
            let world = runtime.world.borrow();
            let world = world.borrow();
            let source = world
                .get::<&ImageSource>(this.entity)
                .map_err(mlua::Error::external)?;
            let size = if let Ok(shape) = world.get::<&PlaneShape>(this.entity) {
                shape.size
            } else {
                world
                    .get::<&RectShape>(this.entity)
                    .map_err(mlua::Error::external)?
                    .size
            };
            Ok(ValueHandle(TrackValue::Color(
                source.pixel_color(point, size),
            )))
        },
    );
    m.add_method(
        "focus_camera",
        |_,
         this,
         (position, target, seconds, ease): (
            mlua::UserDataRef<ValueHandle>,
            mlua::UserDataRef<ValueHandle>,
            Option<f32>,
            Option<String>,
        )| {
            let runtime = this.runtime()?;
            runtime.construction()?;
            let (TrackValue::Vector3(position), TrackValue::Vector3(target)) =
                (&position.0, &target.0)
            else {
                return Err(mlua::Error::runtime(
                    "Camera position and target must be vec3.",
                ));
            };
            if runtime
                .world
                .borrow()
                .borrow()
                .get::<&Camera3D>(this.entity)
                .is_err()
            {
                return Err(mlua::Error::runtime("Focus requires a 3D canvas."));
            }
            let seconds = duration(seconds.unwrap_or(1.0))?;
            let ease = parse_easing(ease)?;
            let handler = Canvas3D::handler(
                runtime.world.borrow().clone(),
                this.entity,
                runtime.animator.borrow().clone(),
            );
            engine(|| {
                TaskHandle(
                    handler
                        .focus_camera(*position, *target)
                        .duration(seconds)
                        .easing(ease)
                        .task(),
                )
            })
        },
    );
}
