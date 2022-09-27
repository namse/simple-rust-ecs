use std::sync::atomic::AtomicU32;

#[derive(Debug, Clone, Copy)]
struct Entity {
    id: u32,
}

static mut ID: AtomicU32 = AtomicU32::new(0);
impl Entity {
    fn new() -> Self {
        Self {
            id: unsafe { ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed) },
        }
    }
    fn add_component<T: Component>(self, component: T) -> Self {
        component.insert(self.id);
        self
    }
}

trait Component {
    fn insert(self, id: u32);
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
const COLLIDE_NONE: Option<Collide> = None;
static mut COLLIDES: [Option<Collide>; 256] = [COLLIDE_NONE; 256];
impl Component for Collide {
    fn insert(self, id: u32) {
        unsafe {
            COLLIDES[id as usize] = Some(self);
        }
    }
}

impl ComponentCombination for &Collide {
    fn filter(entity: &Entity) -> Option<Self> {
        unsafe { COLLIDES.get(entity.id as usize).and_then(|x| x.as_ref()) }
    }
}
impl ComponentCombination for &mut Collide {
    fn filter(entity: &Entity) -> Option<Self> {
        unsafe {
            COLLIDES
                .get_mut(entity.id as usize)
                .and_then(|x| x.as_mut())
        }
    }
}

struct MoveTo {}
impl MoveTo {
    fn move_to(&self) {
        println!("move_to");
    }
}
const MOVE_TO_NONE: Option<MoveTo> = None;
static mut MOVE_TOS: [Option<MoveTo>; 256] = [MOVE_TO_NONE; 256];
impl Component for MoveTo {
    fn insert(self, id: u32) {
        unsafe {
            MOVE_TOS[id as usize] = Some(self);
        }
    }
}
impl ComponentCombination for &MoveTo {
    fn filter(entity: &Entity) -> Option<Self> {
        unsafe { MOVE_TOS.get(entity.id as usize).and_then(|x| x.as_ref()) }
    }
}
impl ComponentCombination for &mut MoveTo {
    fn filter(entity: &Entity) -> Option<Self> {
        unsafe {
            MOVE_TOS
                .get_mut(entity.id as usize)
                .and_then(|x| x.as_mut())
        }
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

    let collide_mut_with_move_to = get_components::<(&mut Collide, &MoveTo)>(&entities);
    println!(
        "-Collide mut with MoveTo- {}",
        collide_mut_with_move_to.len()
    );
    for (collide, move_to) in collide_mut_with_move_to {
        collide.collide();
        move_to.move_to();
    }

    let mut app = App::new();
    app.add_system(simple_system);
    app.add_system(simple_system2);
    app.add_system(simple_system3);

    app.run(&entities);
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

impl<'entity, TA: ComponentCombination, TB: ComponentCombination> ComponentCombination
    for (TA, TB)
{
    fn filter(entity: &Entity) -> Option<Self> {
        let a = TA::filter(entity)?;
        let b = TB::filter(entity)?;
        Some((a, b))
    }
}
