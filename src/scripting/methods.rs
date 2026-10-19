use super::runtime::ObjectHandle;
use crate::core::{
    Task, TrackTarget, TrackValue, Trackable,
    objects::{ObjectHandler, Text2DHandler, TextFade, TextShape},
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
    /// Crossfades the current text into the new text.
    /// Returns an animation; call `:play(duration, easing)` to schedule it.
    #[lua]
    pub(crate) fn fade(&self, text: String) -> Task {
        let tween = |target, from, to, duration, implicit| Task::RuntimeTween {
            entity: self.entity(),
            target,
            from,
            to,
            duration,
            easing: Default::default(),
            mask: 1,
            relative: 0,
            implicit,
            rotation: None,
        };

        Task::All(vec![
            tween(
                TrackTarget::property(std::any::TypeId::of::<TextShape>(), TextShape::track(0)),
                TrackValue::String(text.clone()),
                TrackValue::String(text.clone()),
                1.0,
                true,
            ),
            tween(
                TrackTarget::property(std::any::TypeId::of::<TextFade>(), TextFade::track(0)),
                TrackValue::String(text.clone()),
                TrackValue::String(text),
                0.0,
                false,
            ),
            tween(
                TrackTarget::property(std::any::TypeId::of::<TextFade>(), TextFade::track(1)),
                TrackValue::F32(0.0),
                TrackValue::F32(1.0),
                1.0,
                false,
            ),
        ])
    }
}
