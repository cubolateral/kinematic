use kinematic::prelude::*;

struct UniformScene {
    object: Option<RectHandler>,
}

impl SceneBuilder for UniformScene {
    fn build(&mut self, scene: &mut Scene) {
        let shader = ImageShader::new(
            "#version 330 core\nin vec2 k_uv; out vec4 k_color; uniform float u_progress; void main() { k_color = vec4(u_progress); }",
        );
        let object = rect()
            .shader(&shader)
            .uniform("u_progress", 0.0_f32)
            .build(scene);
        scene.world_2d().add(&object);

        object.set_uniform("u_progress", 0.25_f32);
        object
            .uniform_from("u_progress", 0.0_f32, 1.0_f32)
            .position_x(10.0)
            .duration(2.0)
            .easing(Easing::Linear)
            .play();
        self.object = Some(object);
    }
}

#[test]
fn public_uniform_api_uses_the_normal_timeline() {
    let mut scene = Scene::new();
    let mut builder = UniformScene { object: None };
    scene.build(&mut builder);
    let object = builder.object.unwrap();

    scene.update(1.0);
    assert_eq!(object.get_uniform::<f32>("u_progress"), 0.5);
    assert_eq!(object.get_position().x, 5.0);
}
