use once_cell::sync::OnceCell;
use std::collections::HashMap;
use uuid::Uuid;

struct Entity {
    id: Uuid,
    drop_functions: Vec<Box<dyn FnOnce()>>,
}

impl Entity {
    fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            drop_functions: Vec::new(),
        }
    }
    fn add_component<T: Component>(mut self, component: T) -> Self {
        let id = self.id;
        component.insert(id);
        self.drop_functions.push(Box::new(move || T::drop(id)));
        self
    }
}

impl Drop for Entity {
    fn drop(&mut self) {
        for drop_function in self.drop_functions.drain(..) {
            drop_function();
        }
    }
}

trait Component {
    fn insert(self, id: Uuid);
    fn drop(id: Uuid);
}

fn new_player() -> Entity {
    Entity::new()
        .add_component(Collide {})
        .add_component(MoveTo {})
}

fn new_wall() -> Entity {
    Entity::new().add_component(Collide {})
}

trait ComponentCombination {
    fn filter(entity: &Entity) -> Option<Self>
    where
        Self: Sized;
}

struct Collide {}
impl Collide {
    fn collide(&self) {
        println!("collide");
    }
}
static mut COLLIDES: OnceCell<HashMap<Uuid, Collide>> = OnceCell::new();
impl Component for Collide {
    fn insert(self, id: Uuid) {
        unsafe {
            COLLIDES.get_or_init(|| HashMap::default());
            COLLIDES.get_mut().unwrap().insert(id, self);
        }
    }

    fn drop(id: Uuid) {
        unsafe {
            COLLIDES.get_mut().unwrap().remove(&id);
        }
    }
}

impl ComponentCombination for &Collide {
    fn filter(entity: &Entity) -> Option<Self> {
        unsafe { COLLIDES.get_or_init(|| HashMap::default()).get(&entity.id) }
    }
}
impl ComponentCombination for &mut Collide {
    fn filter(entity: &Entity) -> Option<Self> {
        unsafe {
            COLLIDES.get_or_init(|| HashMap::default());
            COLLIDES.get_mut().unwrap().get_mut(&entity.id)
        }
    }
}

struct MoveTo {}
impl MoveTo {
    fn move_to(&self) {
        println!("move_to");
    }
}
static mut MOVE_TOS: OnceCell<HashMap<Uuid, MoveTo>> = OnceCell::new();
impl Component for MoveTo {
    fn insert(self, id: Uuid) {
        unsafe {
            MOVE_TOS.get_or_init(|| HashMap::default());
            MOVE_TOS.get_mut().unwrap().insert(id, self);
        }
    }

    fn drop(id: Uuid) {
        unsafe {
            MOVE_TOS.get_mut().unwrap().remove(&id);
        }
    }
}
impl ComponentCombination for &MoveTo {
    fn filter(entity: &Entity) -> Option<Self> {
        unsafe { MOVE_TOS.get_or_init(|| HashMap::default()).get(&entity.id) }
    }
}
impl ComponentCombination for &mut MoveTo {
    fn filter(entity: &Entity) -> Option<Self> {
        unsafe {
            MOVE_TOS.get_or_init(|| HashMap::default());
            MOVE_TOS.get_mut().unwrap().get_mut(&entity.id)
        }
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
    fn add_system<'a, T, F>(&'a mut self, system_func: F)
    where
        F: Fn(Vec<T>) + 'static,
        T: ComponentCombination,
    {
        let wrapped_system_func = Box::new(move |entities: &Vec<Entity>| {
            let components = get_components::<T>(entities);
            system_func(components);
        });
        self.systems.push(wrapped_system_func);
    }
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

fn get_components<'entity, T: ComponentCombination>(entities: &Vec<Entity>) -> Vec<T> {
    let mut components = Vec::new();
    for entity in entities {
        if let Some(component) = T::filter(entity) {
            components.push(component);
        }
    }
    components
}

impl<'entity, T0: ComponentCombination, TB: ComponentCombination> ComponentCombination
    for (T0, TB)
{
    fn filter(entity: &Entity) -> Option<Self> {
        let a = T0::filter(entity)?;
        let b = TB::filter(entity)?;
        Some((a, b))
    }
}
