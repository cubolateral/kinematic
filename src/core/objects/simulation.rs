use kinematic_macros::Object;

use crate::core::{
    AnimatorHandle, Scene, SceneWorld, TrackProperty, TrackValueType, Trackable, Tween,
    components::{
        Draw2D, Draw3D, Inspection, RenderContext3D, Simulation, SimulationState2D,
        SimulationState3D, Transform2D, Transform3D,
    },
    objects::{
        HandlerContext, Object2DHandler, Object3DHandler, ObjectBuilderComponent, ObjectHandler,
        global_matrix3d,
    },
};

/// A frame-dependent simulation placed in a two-dimensional scene.
///
/// Create one with [`simulation_2d`]. It behaves like a regular spatial object
/// and contains [`Transform2D`], [`Draw2D`], and scene-node metadata.
#[derive(Object, hecs::Bundle)]
#[object(spatial = "2d", builder = "simulation_2d_base")]
pub struct Simulation2D {
    #[trackable]
    pub simulation: Simulation,
    #[trackable]
    pub transform: Transform2D,
    #[trackable]
    pub draw: Draw2D,
}

impl Default for Simulation2D {
    fn default() -> Self {
        Self {
            simulation: Simulation::default(),
            transform: Transform2D::default(),
            draw: Draw2D {
                on_draw: |world, entity, canvas, _opacity| {
                    world
                        .get::<&mut Simulation>(entity)
                        .unwrap()
                        .draw_2d(world, entity, canvas);
                },
                get_box: |world, entity| {
                    world
                        .get::<&Simulation>(entity)
                        .unwrap()
                        .box_2d(world, entity)
                },
                ..Draw2D::default()
            },
        }
    }
}

impl Simulation2DBuilder {
    /// Sets the state advanced and drawn by this simulation.
    pub fn state<S: SimulationState2D>(mut self, state: S) -> Self {
        self.object.simulation = Simulation::new_2d(state);
        self
    }
}

impl Simulation2DHandler {
    /// Schedules one explicit simulation step at the current scene time.
    ///
    /// Multiple calls in the same frame schedule multiple steps. If automatic
    /// updating is enabled, one explicit call shares its automatic step instead
    /// of adding a duplicate.
    pub fn update(&self) {
        schedule_simulation_update(self);
    }
}

/// A frame-dependent simulation placed in a three-dimensional scene.
///
/// Create one with [`simulation_3d`]. It behaves like a regular spatial object
/// and contains [`Transform3D`], [`Draw3D`], and scene-node metadata.
#[derive(Object, hecs::Bundle)]
#[object(spatial = "3d", builder = "simulation_3d_base")]
pub struct Simulation3D {
    #[trackable]
    pub simulation: Simulation,
    #[trackable]
    pub transform: Transform3D,
    #[trackable]
    pub draw: Draw3D,
}

impl Default for Simulation3D {
    fn default() -> Self {
        Self {
            simulation: Simulation::default(),
            transform: Transform3D::default(),
            draw: Draw3D {
                on_draw: draw_simulation_3d,
                get_box: |world, entity| {
                    world
                        .get::<&Simulation>(entity)
                        .unwrap()
                        .box_3d(world, entity)
                },
                ..Draw3D::default()
            },
        }
    }
}

impl Simulation3DBuilder {
    /// Sets the state advanced and drawn by this simulation.
    pub fn state<S: SimulationState3D>(mut self, state: S) -> Self {
        self.object.simulation = Simulation::new_3d(state);
        self
    }
}

impl Simulation3DHandler {
    /// Schedules one explicit simulation step at the current scene time.
    ///
    /// Multiple calls in the same frame schedule multiple steps. If automatic
    /// updating is enabled, one explicit call shares its automatic step instead
    /// of adding a duplicate.
    pub fn update(&self) {
        schedule_simulation_update(self);
    }
}

impl HandlerContext for Simulation2DHandler {
    type Object = Simulation2D;
}

impl HandlerContext for Simulation3DHandler {
    type Object = Simulation3D;
}

/// Empty tail of a simulation builder's additional component list.
#[doc(hidden)]
pub struct NoSimulationTrackables;

/// One component in a simulation builder's additional component list.
#[doc(hidden)]
pub struct SimulationTrackable<T, Rest> {
    component: T,
    rest: Rest,
}

/// Internal operations for a typed list of additional simulation components.
#[doc(hidden)]
pub trait SimulationTrackableSet: Sized {
    type Fields<Next: HandlerContext>: HandlerContext;

    fn insert(self, world: &SceneWorld, entity: hecs::Entity);

    fn handler_fields<Next: HandlerContext>(
        world: SceneWorld,
        entity: hecs::Entity,
        animator: AnimatorHandle,
        next: Next,
    ) -> Self::Fields<Next>;

    fn copy_components(
        source: &SceneWorld,
        source_entity: hecs::Entity,
        target: &SceneWorld,
        target_entity: hecs::Entity,
    );
}

impl SimulationTrackableSet for NoSimulationTrackables {
    type Fields<Next: HandlerContext> = Next;

    fn insert(self, _world: &SceneWorld, _entity: hecs::Entity) {}

    fn handler_fields<Next: HandlerContext>(
        _world: SceneWorld,
        _entity: hecs::Entity,
        _animator: AnimatorHandle,
        next: Next,
    ) -> Self::Fields<Next> {
        next
    }

    fn copy_components(
        _source: &SceneWorld,
        _source_entity: hecs::Entity,
        _target: &SceneWorld,
        _target_entity: hecs::Entity,
    ) {
    }
}

impl<T, Rest> SimulationTrackableSet for SimulationTrackable<T, Rest>
where
    T: Trackable + hecs::Component + Clone,
    Rest: SimulationTrackableSet,
{
    type Fields<Next: HandlerContext> = T::HandlerFields<Rest::Fields<Next>>;

    fn insert(self, world: &SceneWorld, entity: hecs::Entity) {
        self.rest.insert(world, entity);
        world
            .borrow_mut()
            .insert_one(entity, self.component)
            .expect("A simulation cannot contain the same component type twice.");
        world
            .borrow()
            .get::<&mut Inspection>(entity)
            .expect("Simulation object must contain Inspection metadata.")
            .add_trackable(*T::info());
    }

    fn handler_fields<Next: HandlerContext>(
        world: SceneWorld,
        entity: hecs::Entity,
        animator: AnimatorHandle,
        next: Next,
    ) -> Self::Fields<Next> {
        let next = Rest::handler_fields(std::rc::Rc::clone(&world), entity, animator.clone(), next);
        T::handler_fields(world, entity, animator, next)
    }

    fn copy_components(
        source: &SceneWorld,
        source_entity: hecs::Entity,
        target: &SceneWorld,
        target_entity: hecs::Entity,
    ) {
        Rest::copy_components(source, source_entity, target, target_entity);
        let component = {
            let source = source.borrow();
            let component = source
                .get::<&T>(source_entity)
                .expect("Simulation handler must contain its additional components.");
            (*component).clone()
        };
        target
            .borrow_mut()
            .insert_one(target_entity, component)
            .expect("Copied simulation cannot contain duplicate component types.");
        target
            .borrow()
            .get::<&mut Inspection>(target_entity)
            .expect("Simulation object must contain Inspection metadata.")
            .add_trackable(*T::info());
    }
}

/// Builder for a 2D simulation with a typed list of additional components.
pub struct Simulation2DTrackableBuilder<Extra = NoSimulationTrackables> {
    object: Simulation2D,
    name: String,
    extra: Extra,
}

/// Creates a two-dimensional simulation builder.
pub fn simulation_2d() -> Simulation2DTrackableBuilder {
    Simulation2DTrackableBuilder {
        object: Simulation2D::default(),
        name: "Simulation2D".to_owned(),
        extra: NoSimulationTrackables,
    }
}

impl<Extra> Simulation2DTrackableBuilder<Extra> {
    /// Sets the user-facing object name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Sets the state advanced and drawn by this simulation.
    pub fn state<S: SimulationState2D>(mut self, state: S) -> Self {
        self.object.simulation = Simulation::new_2d(state);
        self
    }

    /// Adds one trackable ECS component to the simulation object.
    pub fn add_trackable<T>(
        self,
        component: T,
    ) -> Simulation2DTrackableBuilder<SimulationTrackable<T, Extra>>
    where
        T: Trackable + hecs::Component + Clone,
    {
        Simulation2DTrackableBuilder {
            object: self.object,
            name: self.name,
            extra: SimulationTrackable {
                component,
                rest: self.extra,
            },
        }
    }
}

impl<Extra: SimulationTrackableSet> Simulation2DTrackableBuilder<Extra> {
    /// Spawns the configured simulation and returns its typed handler.
    pub fn build(self, scene: &mut Scene) -> Simulation2DTrackableHandler<Extra> {
        let base = scene.spawn_object(self.object, self.name);
        let world = base.object_world();
        let entity = base.get_id();
        self.extra.insert(&world, entity);
        Simulation2DTrackableHandler::new(base)
    }
}

impl<Extra> ObjectBuilderComponent<Simulation> for Simulation2DTrackableBuilder<Extra> {
    fn component_mut(&mut self) -> &mut Simulation {
        &mut self.object.simulation
    }
}

impl<Extra> ObjectBuilderComponent<Transform2D> for Simulation2DTrackableBuilder<Extra> {
    fn component_mut(&mut self) -> &mut Transform2D {
        &mut self.object.transform
    }
}

impl<Extra> ObjectBuilderComponent<Draw2D> for Simulation2DTrackableBuilder<Extra> {
    fn component_mut(&mut self) -> &mut Draw2D {
        &mut self.object.draw
    }
}

/// Handler for a 2D simulation and its additional trackable components.
pub struct Simulation2DTrackableHandler<Extra: SimulationTrackableSet = NoSimulationTrackables> {
    base: Simulation2DHandler,
    fields: Extra::Fields<Simulation2DHandler>,
}

impl<Extra: SimulationTrackableSet> Simulation2DTrackableHandler<Extra> {
    fn new(base: Simulation2DHandler) -> Self {
        let fields = Extra::handler_fields(
            base.object_world(),
            base.get_id(),
            base.object_animator(),
            base.clone(),
        );
        Self { base, fields }
    }

    /// Schedules one explicit simulation step at the current scene time.
    pub fn update(&self) {
        schedule_simulation_update(self);
    }

    /// Creates an identical object in the supplied scene.
    pub fn copy(&self, scene: &mut Scene) -> Self {
        let base = self.base.copy(scene);
        Extra::copy_components(
            &self.base.object_world(),
            self.base.get_id(),
            &base.object_world(),
            base.get_id(),
        );
        Self::new(base)
    }
}

impl<Extra: SimulationTrackableSet> Clone for Simulation2DTrackableHandler<Extra> {
    fn clone(&self) -> Self {
        Self::new(self.base.clone())
    }
}

impl<Extra: SimulationTrackableSet> std::ops::Deref for Simulation2DTrackableHandler<Extra> {
    type Target = Extra::Fields<Simulation2DHandler>;

    fn deref(&self) -> &Self::Target {
        &self.fields
    }
}

impl<Extra: SimulationTrackableSet> ObjectHandler for Simulation2DTrackableHandler<Extra> {
    type Object = Simulation2D;

    fn object_world(&self) -> SceneWorld {
        self.base.object_world()
    }
    fn object_animator(&self) -> AnimatorHandle {
        self.base.object_animator()
    }
    fn get_id(&self) -> hecs::Entity {
        self.base.get_id()
    }
    fn get_name(&self) -> String {
        self.base.get_name()
    }
    fn set_name(&self, name: impl Into<String>) {
        self.base.set_name(name);
    }
    fn remove(&self) {
        self.base.remove();
    }
    fn get<T: TrackValueType>(&self, property: TrackProperty<T>) -> T {
        self.base.get(property)
    }
    fn animate<T: TrackValueType>(&self, property: TrackProperty<T>, to: T) -> Tween<Self::Object> {
        self.base.animate(property, to)
    }
    fn animate_from<T: TrackValueType>(
        &self,
        property: TrackProperty<T>,
        from: T,
        to: T,
    ) -> Tween<Self::Object> {
        self.base.animate_from(property, from, to)
    }
    fn save(&self) {
        self.base.save();
    }
    fn restore(&self) -> Tween<Self::Object> {
        self.base.restore()
    }
}

impl<Extra: SimulationTrackableSet> Object2DHandler for Simulation2DTrackableHandler<Extra> {
    fn get_box(&self) -> glam::Vec2 {
        self.base.get_box()
    }
    fn get_global_position(&self) -> glam::Vec2 {
        self.base.get_global_position()
    }
    fn get_global_rotation(&self) -> f32 {
        self.base.get_global_rotation()
    }
    fn get_global_scale(&self) -> glam::Vec2 {
        self.base.get_global_scale()
    }
    fn get_global_opacity(&self) -> f32 {
        self.base.get_global_opacity()
    }
}

/// Builder for a 3D simulation with a typed list of additional components.
pub struct Simulation3DTrackableBuilder<Extra = NoSimulationTrackables> {
    object: Simulation3D,
    name: String,
    extra: Extra,
}

/// Creates a three-dimensional simulation builder.
pub fn simulation_3d() -> Simulation3DTrackableBuilder {
    Simulation3DTrackableBuilder {
        object: Simulation3D::default(),
        name: "Simulation3D".to_owned(),
        extra: NoSimulationTrackables,
    }
}

impl<Extra> Simulation3DTrackableBuilder<Extra> {
    /// Sets the user-facing object name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Sets the state advanced and drawn by this simulation.
    pub fn state<S: SimulationState3D>(mut self, state: S) -> Self {
        self.object.simulation = Simulation::new_3d(state);
        self
    }

    /// Adds one trackable ECS component to the simulation object.
    pub fn add_trackable<T>(
        self,
        component: T,
    ) -> Simulation3DTrackableBuilder<SimulationTrackable<T, Extra>>
    where
        T: Trackable + hecs::Component + Clone,
    {
        Simulation3DTrackableBuilder {
            object: self.object,
            name: self.name,
            extra: SimulationTrackable {
                component,
                rest: self.extra,
            },
        }
    }
}

impl<Extra: SimulationTrackableSet> Simulation3DTrackableBuilder<Extra> {
    /// Spawns the configured simulation and returns its typed handler.
    pub fn build(self, scene: &mut Scene) -> Simulation3DTrackableHandler<Extra> {
        let base = scene.spawn_object(self.object, self.name);
        let world = base.object_world();
        let entity = base.get_id();
        self.extra.insert(&world, entity);
        Simulation3DTrackableHandler::new(base)
    }
}

impl<Extra> ObjectBuilderComponent<Simulation> for Simulation3DTrackableBuilder<Extra> {
    fn component_mut(&mut self) -> &mut Simulation {
        &mut self.object.simulation
    }
}

impl<Extra> ObjectBuilderComponent<Transform3D> for Simulation3DTrackableBuilder<Extra> {
    fn component_mut(&mut self) -> &mut Transform3D {
        &mut self.object.transform
    }
}

impl<Extra> ObjectBuilderComponent<Draw3D> for Simulation3DTrackableBuilder<Extra> {
    fn component_mut(&mut self) -> &mut Draw3D {
        &mut self.object.draw
    }
}

/// Handler for a 3D simulation and its additional trackable components.
pub struct Simulation3DTrackableHandler<Extra: SimulationTrackableSet = NoSimulationTrackables> {
    base: Simulation3DHandler,
    fields: Extra::Fields<Simulation3DHandler>,
}

impl<Extra: SimulationTrackableSet> Simulation3DTrackableHandler<Extra> {
    fn new(base: Simulation3DHandler) -> Self {
        let fields = Extra::handler_fields(
            base.object_world(),
            base.get_id(),
            base.object_animator(),
            base.clone(),
        );
        Self { base, fields }
    }

    /// Schedules one explicit simulation step at the current scene time.
    pub fn update(&self) {
        schedule_simulation_update(self);
    }

    /// Creates an identical object in the supplied scene.
    pub fn copy(&self, scene: &mut Scene) -> Self {
        let base = self.base.copy(scene);
        Extra::copy_components(
            &self.base.object_world(),
            self.base.get_id(),
            &base.object_world(),
            base.get_id(),
        );
        Self::new(base)
    }
}

impl<Extra: SimulationTrackableSet> Clone for Simulation3DTrackableHandler<Extra> {
    fn clone(&self) -> Self {
        Self::new(self.base.clone())
    }
}

impl<Extra: SimulationTrackableSet> std::ops::Deref for Simulation3DTrackableHandler<Extra> {
    type Target = Extra::Fields<Simulation3DHandler>;

    fn deref(&self) -> &Self::Target {
        &self.fields
    }
}

impl<Extra: SimulationTrackableSet> ObjectHandler for Simulation3DTrackableHandler<Extra> {
    type Object = Simulation3D;

    fn object_world(&self) -> SceneWorld {
        self.base.object_world()
    }
    fn object_animator(&self) -> AnimatorHandle {
        self.base.object_animator()
    }
    fn get_id(&self) -> hecs::Entity {
        self.base.get_id()
    }
    fn get_name(&self) -> String {
        self.base.get_name()
    }
    fn set_name(&self, name: impl Into<String>) {
        self.base.set_name(name);
    }
    fn remove(&self) {
        self.base.remove();
    }
    fn get<T: TrackValueType>(&self, property: TrackProperty<T>) -> T {
        self.base.get(property)
    }
    fn animate<T: TrackValueType>(&self, property: TrackProperty<T>, to: T) -> Tween<Self::Object> {
        self.base.animate(property, to)
    }
    fn animate_from<T: TrackValueType>(
        &self,
        property: TrackProperty<T>,
        from: T,
        to: T,
    ) -> Tween<Self::Object> {
        self.base.animate_from(property, from, to)
    }
    fn save(&self) {
        self.base.save();
    }
    fn restore(&self) -> Tween<Self::Object> {
        self.base.restore()
    }
}

impl<Extra: SimulationTrackableSet> Object3DHandler for Simulation3DTrackableHandler<Extra> {}

/// Schedules one explicit step for a custom simulation object.
///
/// The object must contain a [`Simulation`] component. Multiple calls at the
/// same scene time schedule multiple steps.
pub fn schedule_simulation_update(handler: &impl ObjectHandler) {
    let animator = handler.object_animator();
    animator.assert_finite_scope();
    handler
        .object_world()
        .borrow()
        .get::<&mut Simulation>(handler.get_id())
        .expect("Simulation handler must contain a Simulation component.")
        .schedule_update(animator.time());
}

fn draw_simulation_3d(
    world: &hecs::World,
    entity: hecs::Entity,
    context: &mut RenderContext3D<'_>,
) -> Result<(), String> {
    let previous = context.set_current_transform(global_matrix3d(world, entity));
    let result = world
        .get::<&mut Simulation>(entity)
        .unwrap()
        .draw_3d(world, entity, context);
    context.set_current_transform(previous);
    result
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::prelude::*;

    #[derive(Clone)]
    struct Counter {
        steps: u64,
        observed: Arc<Mutex<u64>>,
    }

    #[derive(Clone)]
    struct Solid;

    #[derive(Clone, Trackable)]
    struct Parameters {
        #[track]
        rule: u32,
    }

    #[derive(Clone, Trackable)]
    struct Appearance {
        #[track]
        color: Color,
    }

    #[derive(Clone)]
    struct RuleRecorder {
        rules: Vec<u32>,
        observed: Arc<Mutex<Vec<u32>>>,
    }

    impl SimulationState for RuleRecorder {
        fn on_update(&mut self, context: &SimulationContext<'_>) {
            self.rules.push(context.get::<Parameters>().rule);
        }
    }

    impl SimulationState2D for RuleRecorder {
        fn on_draw(
            &mut self,
            _world: &hecs::World,
            _entity: hecs::Entity,
            _canvas: &skia_safe::Canvas,
        ) {
            *self.observed.lock().unwrap() = self.rules.clone();
        }
    }

    impl SimulationState for Solid {
        fn on_update(&mut self, _context: &SimulationContext<'_>) {}
    }

    impl SimulationState2D for Solid {
        fn on_draw(
            &mut self,
            _world: &hecs::World,
            _entity: hecs::Entity,
            canvas: &skia_safe::Canvas,
        ) {
            canvas.draw_rect(
                skia_safe::Rect::from_xywh(-4.0, -4.0, 8.0, 8.0),
                &skia_safe::Paint::new(skia_safe::colors::WHITE, None),
            );
        }

        fn get_box(&self, _world: &hecs::World, _entity: hecs::Entity) -> Vector2 {
            Vector2::splat(8.0)
        }
    }

    impl SimulationState for Counter {
        fn on_update(&mut self, _context: &SimulationContext<'_>) {
            self.steps += 1;
            *self.observed.lock().unwrap() = self.steps;
        }
    }

    impl SimulationState2D for Counter {
        fn on_draw(
            &mut self,
            _world: &hecs::World,
            _entity: hecs::Entity,
            _canvas: &skia_safe::Canvas,
        ) {
            *self.observed.lock().unwrap() = self.steps;
        }

        fn get_box(&self, _world: &hecs::World, _entity: hecs::Entity) -> glam::Vec2 {
            glam::Vec2::splat(10.0)
        }
    }

    fn draw(scene: &Scene) {
        let mut surface = skia_safe::surfaces::raster_n32_premul((16, 16)).unwrap();
        scene.draw(surface.canvas());
    }

    fn rule_scene(observed: Arc<Mutex<Vec<u32>>>) -> Scene {
        struct Setup(Arc<Mutex<Vec<u32>>>);

        impl SceneBuilder for Setup {
            fn build(&mut self, scene: &mut Scene) {
                let simulation = simulation_2d()
                    .state(RuleRecorder {
                        rules: Vec::new(),
                        observed: Arc::clone(&self.0),
                    })
                    .add_trackable(Parameters { rule: 30 })
                    .add_trackable(Appearance { color: Color::CYAN })
                    .build(scene);
                assert_eq!(simulation.get_rule(), 30);
                simulation.set_rule(30_u32);
                assert_eq!(simulation.get_color(), Color::CYAN);
                {
                    let world = scene.get_world();
                    let inspection = world.get::<&Inspection>(simulation.get_id()).unwrap();
                    let components: Vec<_> = inspection
                        .trackables(&world, simulation.get_id())
                        .map(|component| component.name)
                        .collect();
                    assert!(components.contains(&"Parameters"));
                    assert!(components.contains(&"Appearance"));
                }
                scene.get_world_2d().add(&simulation);
                scene.wait(1.0);
                simulation.rule(90).immediate();
                simulation.color(Color::MAGENTA).immediate();
                scene.wait(1.0);
            }
        }

        let mut scene = Scene::new();
        scene.build(&mut Setup(observed));
        scene.set_fps(10);
        scene
    }

    fn observed_rules(scene: &Scene, observed: &Arc<Mutex<Vec<u32>>>) -> Vec<u32> {
        draw(scene);
        observed.lock().unwrap().clone()
    }

    #[test]
    fn trackable_rule_is_sampled_per_frame_for_steps_jumps_and_backward_seeks() {
        let sequential_observed = Arc::new(Mutex::new(Vec::new()));
        let sequential = rule_scene(Arc::clone(&sequential_observed));
        for frame in 0..=15 {
            sequential.update(frame as f32 / 10.0);
        }
        let sequential_rules = observed_rules(&sequential, &sequential_observed);
        assert_eq!(sequential_rules.len(), 16);
        assert_eq!(&sequential_rules[..10], &[30; 10]);
        assert_eq!(&sequential_rules[10..], &[90; 6]);

        let jump_observed = Arc::new(Mutex::new(Vec::new()));
        let jump = rule_scene(Arc::clone(&jump_observed));
        jump.update(1.5);
        assert_eq!(observed_rules(&jump, &jump_observed), sequential_rules);

        jump.update(0.5);
        assert_eq!(observed_rules(&jump, &jump_observed), vec![30; 6]);
        jump.update(1.5);
        assert_eq!(observed_rules(&jump, &jump_observed), sequential_rules);
    }

    #[test]
    fn scene_replays_the_historical_auto_update_track_when_seeking() {
        let observed = Arc::new(Mutex::new(0));
        let mut scene = Scene::new();
        struct Setup(Arc<Mutex<u64>>);

        impl SceneBuilder for Setup {
            fn build(&mut self, scene: &mut Scene) {
                let simulation = simulation_2d()
                    .state(Counter {
                        steps: 0,
                        observed: Arc::clone(&self.0),
                    })
                    .position(vec2(100.0, 0.0))
                    .build(scene);
                scene.get_world_2d().add(&simulation);
                scene.wait(0.3);
                simulation.auto_update(false).immediate();
                scene.wait(0.3);
            }
        }

        scene.build(&mut Setup(Arc::clone(&observed)));
        scene.set_fps(10);

        scene.update(0.6);
        draw(&scene);
        assert_eq!(*observed.lock().unwrap(), 3);

        scene.update(0.1);
        draw(&scene);
        assert_eq!(*observed.lock().unwrap(), 2);

        scene.update(0.6);
        draw(&scene);
        assert_eq!(*observed.lock().unwrap(), 3);
    }

    #[test]
    fn handler_can_schedule_multiple_steps_in_one_frame() {
        let observed = Arc::new(Mutex::new(0));
        let mut scene = Scene::new();
        struct Setup(Arc<Mutex<u64>>);

        impl SceneBuilder for Setup {
            fn build(&mut self, scene: &mut Scene) {
                let simulation = simulation_2d()
                    .state(Counter {
                        steps: 0,
                        observed: Arc::clone(&self.0),
                    })
                    .build(scene);
                scene.get_world_2d().add(&simulation);
                simulation.auto_update(false).immediate();
                for _ in 0..4 {
                    simulation.update();
                }
            }
        }

        scene.build(&mut Setup(Arc::clone(&observed)));
        scene.update(0.0);
        draw(&scene);

        assert_eq!(*observed.lock().unwrap(), 4);
    }

    #[test]
    fn simulation_2d_inherits_group_transform_and_composited_opacity() {
        let mut scene = Scene::new();
        struct Setup;

        impl SceneBuilder for Setup {
            fn build(&mut self, scene: &mut Scene) {
                let group = group_2d()
                    .position(vec2(4.0, 0.0))
                    .opacity(0.5)
                    .build(scene);
                let simulation = simulation_2d()
                    .state(Solid)
                    .position(vec2(4.0, 0.0))
                    .opacity(0.5)
                    .build(scene);
                group.add(&simulation);
                scene.get_world_2d().add(&group);
            }
        }

        scene.build(&mut Setup);
        scene.update(0.0);
        let image_info = skia_safe::ImageInfo::new(
            (32, 32),
            skia_safe::ColorType::RGBA8888,
            skia_safe::AlphaType::Premul,
            None,
        );
        let mut surface = skia_safe::surfaces::raster(&image_info, None, None).unwrap();
        surface.canvas().clear(skia_safe::colors::TRANSPARENT);
        surface.canvas().translate((16.0, 16.0));
        scene.draw(surface.canvas());

        let pixels = surface.peek_pixels().unwrap();
        let inside = pixels.get_color((24, 16));
        assert_eq!(inside.r(), 255);
        assert!((63..=64).contains(&inside.a()));
        assert_eq!(pixels.get_color((16, 16)).a(), 0);
    }
}
