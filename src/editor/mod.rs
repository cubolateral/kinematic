use crate::core::Project;

pub(crate) struct Editor {
    project: Project,
}

impl Editor {
    pub fn new(project: Project) -> Self {
        println!("Project initialized: {}", project.name);
        Self { project }
    }

    pub fn update(&mut self, _dt: f32) {}

    pub fn draw(&mut self) {}
}
