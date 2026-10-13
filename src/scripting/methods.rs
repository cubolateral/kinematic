use super::runtime::ObjectHandle;
use crate::core::{
    Task, TrackValue, Trackable,
    components::Draw2D,
    objects::{ObjectHandler, Text2DHandler, TextShape},
};
use mlua::{Lua, MultiValue, Result, Value};

pub(crate) struct LuaMethod {
    pub name: &'static str,
    pub annotation: &'static str,
    pub documentation: &'static str,
    pub invoke: fn(&Lua, &ObjectHandle, MultiValue) -> Result<Value>,
}

#[kinematic_macros::lua_methods]
impl Text2DHandler {
    /// Replaces the text and fades it from transparent to its current opacity.
    /// Returns an animation; call `:play(duration, easing)` to schedule it.
    #[lua]
    pub(crate) fn fade(&self, text: String) -> Task {
        let opacity = self
            .object_world()
            .borrow()
            .get::<&Draw2D>(self.entity())
            .unwrap()
            .opacity;
        Task::All(vec![
            Task::RuntimeTween {
                entity: self.entity(),
                target: crate::core::TrackTarget::property(
                    std::any::TypeId::of::<TextShape>(),
                    TextShape::track(0),
                ),
                from: TrackValue::String(text.clone()),
                to: TrackValue::String(text),
                duration: 0.0,
                easing: Default::default(),
                mask: 1,
                relative: 0,
                implicit: false,
                rotation: None,
            },
            Task::RuntimeTween {
                entity: self.entity(),
                target: crate::core::TrackTarget::property(
                    std::any::TypeId::of::<crate::core::components::Draw2D>(),
                    crate::core::components::Draw2D::opacity_property().info(),
                ),
                from: TrackValue::F32(0.0),
                to: TrackValue::F32(opacity),
                duration: 1.0,
                easing: Default::default(),
                mask: 1,
                relative: 0,
                implicit: false,
                rotation: None,
            },
        ])
    }
}
