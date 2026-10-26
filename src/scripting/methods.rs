use super::runtime::ObjectHandle;
use crate::core::{
    Task, TrackTarget, TrackValue, Trackable,
    objects::{
        Latex2DHandler, Latex2DShape, Latex3DHandler, Latex3DShape, ObjectHandler, Text2DHandler,
        Text3DHandler, Text3DShape, TextFade, TextShape,
    },
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
    /// Moves matching glyphs and fades added or removed glyphs.
    /// Returns an animation; call `:play(duration, easing)` to schedule it.
    #[lua]
    pub(crate) fn morph(&self, text: String) -> Task {
        text_transition::<TextShape>(self.entity(), text, true)
    }

    /// Crossfades the current text into the new text.
    /// Returns an animation; call `:play(duration, easing)` to schedule it.
    #[lua]
    pub(crate) fn fade(&self, text: String) -> Task {
        text_transition::<TextShape>(self.entity(), text, false)
    }
}

#[kinematic_macros::lua_methods]
impl Latex2DHandler {
    /// Moves matching glyphs and fades added or removed glyphs.
    /// Returns an animation; call `:play(duration, easing)` to schedule it.
    #[lua]
    pub(crate) fn morph(&self, text: String) -> Task {
        text_transition::<Latex2DShape>(self.entity(), text, true)
    }

    /// Crossfades the current formula into the new formula.
    /// Returns an animation; call `:play(duration, easing)` to schedule it.
    #[lua]
    pub(crate) fn fade(&self, text: String) -> Task {
        text_transition::<Latex2DShape>(self.entity(), text, false)
    }
}

#[kinematic_macros::lua_methods]
impl Latex3DHandler {
    /// Moves matching glyphs and fades added or removed glyphs.
    /// Returns an animation; call `:play(duration, easing)` to schedule it.
    #[lua]
    pub(crate) fn morph(&self, text: String) -> Task {
        text_transition::<Latex3DShape>(self.entity(), text, true)
    }

    /// Crossfades the current formula into the new formula.
    /// Returns an animation; call `:play(duration, easing)` to schedule it.
    #[lua]
    pub(crate) fn fade(&self, text: String) -> Task {
        text_transition::<Latex3DShape>(self.entity(), text, false)
    }
}

#[kinematic_macros::lua_methods]
impl Text3DHandler {
    /// Moves matching glyphs and fades added or removed glyphs.
    /// Returns an animation; call `:play(duration, easing)` to schedule it.
    #[lua]
    pub(crate) fn morph(&self, text: String) -> Task {
        text_transition::<Text3DShape>(self.entity(), text, true)
    }
}

pub(super) fn text_transition<T: Trackable + 'static>(
    entity: hecs::Entity,
    text: String,
    morph: bool,
) -> Task {
    let tween = |target, from, to, duration, implicit| Task::RuntimeTween {
        entity,
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
            TrackTarget::property(std::any::TypeId::of::<TextFade>(), TextFade::track(3)),
            TrackValue::Bool(morph),
            TrackValue::Bool(morph),
            0.0,
            false,
        ),
        tween(
            TrackTarget::property(std::any::TypeId::of::<T>(), T::track(0)),
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
        tween(
            TrackTarget::property(std::any::TypeId::of::<TextFade>(), TextFade::track(2)),
            TrackValue::Bool(true),
            TrackValue::Bool(false),
            1.0,
            false,
        ),
    ])
}
