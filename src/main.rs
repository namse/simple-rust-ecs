use rustc_hash::FxHashMap;
use std::any::{Any, TypeId};
use uuid::Uuid;

struct Entity {
    id: Uuid,
    components: Vec<Box<dyn Any>>,
}

impl Entity {
    fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            components: Vec::new(),
        }
    }
    fn add_component<T: 'static>(mut self, component: T) -> Self {
        self.components.push(Box::new(component));
        self
    }
}

fn new_player() -> Entity {
    Entity::new()
        .add_component(MoveTo {})
        .add_component(MoveTo {})
        .add_component(MoveTo {})
        .add_component(MoveTo {})
        .add_component(MoveTo {})
        .add_component(Collide {})
}

fn new_wall() -> Entity {
    Entity::new().add_component(Collide {})
}

trait ComponentCombination<'a> {
    fn filter(entity: &'a Entity) -> Option<Self>
    where
        Self: Sized;
}

trait ComponentCombinationMut<'a> {
    fn filter(entity: &'a mut Entity) -> Option<Self>
    where
        Self: Sized;
}

struct Collide {}
impl Collide {
    fn collide(&self) {
        println!("collide");
    }
}
impl<'a> ComponentCombination<'a> for &'a Collide {
    fn filter(entity: &'a Entity) -> Option<Self> {
        entity
            .components
            .iter()
            .find(|c| c.is::<Collide>())
            .map(|c| c.downcast_ref::<Collide>().unwrap())
    }
}
impl<'a> ComponentCombinationMut<'a> for &'a mut Collide {
    fn filter(entity: &'a mut Entity) -> Option<Self> {
        entity
            .components
            .iter_mut()
            .find(|c| c.is::<Collide>())
            .map(|c| c.downcast_mut::<Collide>().unwrap())
    }
}

struct MoveTo {}
impl MoveTo {
    fn move_to(&self) {
        println!("move_to");
    }
}
impl<'a> ComponentCombination<'a> for &'a MoveTo {
    fn filter(entity: &'a Entity) -> Option<Self> {
        entity
            .components
            .iter()
            .find(|c| c.is::<MoveTo>())
            .map(|c| c.downcast_ref::<MoveTo>().unwrap())
    }
}
impl<'a> ComponentCombinationMut<'a> for &'a mut MoveTo {
    fn filter(entity: &'a mut Entity) -> Option<Self> {
        entity
            .components
            .iter_mut()
            .find(|c| c.is::<MoveTo>())
            .map(|c| c.downcast_mut::<MoveTo>().unwrap())
    }
}
fn main() {
    // let entities = vec![new_player(), new_wall()];

    // let collides = get_components::<&Collide>(&entities);
    // println!("-Collide- {}", collides.len());
    // for collide in collides {
    //     collide.collide();
    // }

    // let move_tos = get_components::<&MoveTo>(&entities);
    // println!("-MoveTo- {}", move_tos.len());
    // for move_to in move_tos {
    //     move_to.move_to();
    // }

    // let collide_with_move_to = get_components::<(&Collide, &MoveTo)>(&entities);
    // println!("-Collide with MoveTo- {}", collide_with_move_to.len());
    // for (collide, move_to) in collide_with_move_to {
    //     collide.collide();
    //     move_to.move_to();
    // }

    // let collide_mut_with_move_to = get_components::<(&mut Collide, &MoveTo)>(&entities);
    // println!(
    //     "-Collide mut with MoveTo- {}",
    //     collide_mut_with_move_to.len()
    // );
    // for (collide, move_to) in collide_mut_with_move_to {
    //     collide.collide();
    //     move_to.move_to();
    // }

    // let mut app = App::new();
    // app.add_system(simple_system);
    // app.add_system(simple_system2);
    // app.add_system(simple_system3);

    // app.run(&entities);
    for trial in 0..10 {
        let mut entities = vec![];
        for _ in 0..100_000 {
            entities.push(new_player());
        }

        let now = std::time::Instant::now();
        let collides = get_components::<&Collide>(&entities);
        println!("-Collide- {}", collides.len());
        println!("trial {trial} time: {:?}", now.elapsed());
    }
}

struct App {
    systems: Vec<Box<dyn Fn(&Vec<Entity>)>>,
}

impl App {
    fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }
    // fn add_system<'a, T, F>(&'a mut self, system_func: F)
    // where
    //     F: Fn(Vec<T>) + 'static,
    //     T: ComponentCombination,
    // {
    //     let wrapped_system_func = Box::new(move |entities: &Vec<Entity>| {
    //         let components = get_components::<T>(entities);
    //         system_func(components);
    //     });
    //     self.systems.push(wrapped_system_func);
    // }
    fn run(&self, entities: &Vec<Entity>) {
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

impl<'entity, T0: ComponentCombination<'entity>, TB: ComponentCombination<'entity>>
    ComponentCombination<'entity> for (T0, TB)
{
    fn filter(entity: &'entity Entity) -> Option<Self> {
        let a = T0::filter(entity)?;
        let b = TB::filter(entity)?;
        Some((a, b))
    }
}
