use std::any::Any;

struct Entity {
    components: Vec<Box<dyn Any>>,
}

impl Entity {
    fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }
    fn add_component<T: Any>(mut self, component: T) -> Self {
        self.components.push(Box::new(component));
        self
    }
}

fn new_player() -> Entity {
    Entity::new()
        .add_component(Collide {})
        .add_component(MoveTo {})
}

fn new_wall() -> Entity {
    Entity::new().add_component(Collide {})
}

trait ComponentCombination<'entity> {
    fn filter(entity: &'entity Entity) -> Option<Self>
    where
        Self: Sized;
}

struct Collide {}
impl Collide {
    fn collide(&self) {
        println!("collide");
    }
}
impl<'entity> ComponentCombination<'entity> for &'entity Collide {
    fn filter(entity: &'entity Entity) -> Option<Self> {
        for component in &entity.components {
            if let Some(component) = component.downcast_ref::<Collide>() {
                return Some(component);
            }
        }
        None
    }
}

struct MoveTo {}
impl MoveTo {
    fn move_to(&self) {
        println!("move_to");
    }
}
impl<'entity> ComponentCombination<'entity> for &'entity MoveTo {
    fn filter(entity: &'entity Entity) -> Option<Self> {
        for component in &entity.components {
            if let Some(component) = component.downcast_ref::<MoveTo>() {
                return Some(component);
            }
        }
        None
    }
}
fn main() {
    let entities = vec![new_player(), new_wall()];

    let collides = get_components::<&Collide>(&entities);
    println!("-Collide- {}", collides.len());
    for collide in collides {
        collide.collide();
    }

    let move_tos = get_components::<&MoveTo>(&entities);
    println!("-MoveTo- {}", move_tos.len());
    for move_to in move_tos {
        move_to.move_to();
    }

    let collide_with_move_to = get_components::<(&Collide, &MoveTo)>(&entities);
    println!("-Collide with MoveTo- {}", collide_with_move_to.len());
    for (collide, move_to) in collide_with_move_to {
        collide.collide();
        move_to.move_to();
    }

    let mut app = App::new();
    app.add_system(simple_system);
    app.add_system(simple_system2);
    app.add_system(simple_system3);

    app.run(&entities);
}

struct App<'entity> {
    systems: Vec<Box<dyn Fn(&'entity Vec<Entity>)>>,
}

impl<'entity> App<'entity> {
    fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }
    fn add_system<'a, T, F>(&'a mut self, system_func: F)
    where
        F: Fn(Vec<T>) + 'static,
        T: ComponentCombination<'entity>,
    {
        let wrapped_system_func = Box::new(move |entities: &'entity Vec<Entity>| {
            let components = get_components::<T>(entities);
            system_func(components);
        });
        self.systems.push(wrapped_system_func);
    }
    fn run(&self, entities: &'entity Vec<Entity>) {
        for system in &self.systems {
            system(entities);
        }
    }
}

fn simple_system(collides: Vec<&Collide>) {
    println!("simple_system");
    for collide in collides {
        collide.collide();
    }
}

fn simple_system2(move_tos: Vec<&MoveTo>) {
    println!("simple_system2");
    for move_to in move_tos {
        move_to.move_to();
    }
}

fn simple_system3(tuples: Vec<(&Collide, &MoveTo)>) {
    println!("simple_system3");
    for (collide, move_to) in tuples {
        collide.collide();
        move_to.move_to();
    }
}

fn get_components<'entity, T: ComponentCombination<'entity>>(
    entities: &'entity Vec<Entity>,
) -> Vec<T> {
    let mut components = Vec::new();
    for entity in entities {
        if let Some(component) = T::filter(entity) {
            components.push(component);
        }
    }
    components
}

impl<'entity, TA: ComponentCombination<'entity>, TB: ComponentCombination<'entity>>
    ComponentCombination<'entity> for (TA, TB)
{
    fn filter(entity: &'entity Entity) -> Option<Self> {
        let a = TA::filter(entity)?;
        let b = TB::filter(entity)?;
        Some((a, b))
    }
}
