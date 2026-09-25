use kinematic::prelude::*;

#[derive(Object, Node)]
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

#[test]
fn custom_container_manages_children_and_sibling_order() {
    let mut scene = Scene::new();
    let parent = custom_container().build(&mut scene);
    let nested = custom_container().build(&mut scene);
    let first = rect().build(&mut scene);
    let circle = circle().build(&mut scene);
    let last = rect().build(&mut scene);
    let nested_child = rect().build(&mut scene);

    parent.add(&first);
    parent.add(&circle);
    parent.insert(&last, 1);
    parent.add(&nested);
    nested.add(&nested_child);

    assert_eq!(
        parent.children(),
        vec![
            first.entity(),
            last.entity(),
            circle.entity(),
            nested.entity()
        ]
    );
    assert_eq!(
        parent.first_child::<Rect>().unwrap().entity(),
        first.entity()
    );
    assert_eq!(parent.last_child::<Rect>().unwrap().entity(), last.entity());
    assert_eq!(
        parent
            .all_children::<Rect>()
            .into_iter()
            .map(|handler| handler.entity())
            .collect::<Vec<_>>(),
        vec![first.entity(), last.entity()]
    );
    assert!(parent.first_child::<Line2D>().is_none());
    assert_eq!(
        nested.ancestor::<CustomContainer>().unwrap().entity(),
        parent.entity()
    );
    assert_eq!(nested.parent(), Some(parent.entity()));
    assert_eq!(
        parent.first_child_recursively::<Rect>().unwrap().entity(),
        first.entity()
    );
    assert_eq!(
        parent.last_child_recursively::<Rect>().unwrap().entity(),
        nested_child.entity()
    );
    assert_eq!(
        parent
            .all_children_recursively::<Rect>()
            .into_iter()
            .map(|handler| handler.entity())
            .collect::<Vec<_>>(),
        vec![first.entity(), last.entity(), nested_child.entity()]
    );

    first.move_to_top();
    first.move_down();
    first.move_to_bottom();
    first.move_up();
    first.move_to(2);
    assert_eq!(parent.children()[2], first.entity());

    parent.remove_children();
}
