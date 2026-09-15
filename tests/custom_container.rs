use kinematic::prelude::*;

#[derive(Object, Container)]
#[object(spatial = "2d", builder = "custom_container")]
struct CustomContainer {
    #[trackable]
    transform: Transform2D,
    #[trackable]
    draw: Draw2D,
}

impl Default for CustomContainer {
    fn default() -> Self {
        Self {
            transform: Default::default(),
            draw: Default::default(),
        }
    }
}

#[test]
fn custom_container_returns_typed_direct_children() {
    let mut scene = Scene::new();
    let container = custom_container().build(&mut scene);
    let rectangle = rect().build(&mut scene);
    let circle = circle().build(&mut scene);
    container.add(&rectangle);
    container.add(&circle);

    assert_eq!(
        container.children(),
        vec![rectangle.entity(), circle.entity()]
    );
    assert_eq!(container.get_child_entity(0), Ok(rectangle.entity()));
    assert_eq!(
        container.get_child_entity(2),
        Err(ChildError::NotFound { index: 2, len: 2 })
    );

    assert_eq!(
        container.get_child::<Rect>(0).unwrap().entity(),
        rectangle.entity()
    );
    assert_eq!(
        container.get_child::<Circle>(1).unwrap().entity(),
        circle.entity()
    );

    assert!(matches!(
        container.get_child::<Circle>(0),
        Err(ChildError::TypeMismatch {
            index: 0,
            expected: "Circle",
            actual: "Rect",
        })
    ));
    assert!(matches!(
        container.get_child::<Rect>(2),
        Err(ChildError::NotFound { index: 2, len: 2 })
    ));
}
