use crate::core::SceneBuilder;

pub struct Project {
    pub name: &'static str,
    pub resolution: (u32, u32),
    pub scene: Box<dyn SceneBuilder>,
}
