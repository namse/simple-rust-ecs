use once_cell::sync::OnceCell;
use sparseset::SparseSet;
use std::sync::atomic::AtomicUsize;

struct Entity {
    id: usize,
    drop_functions: Vec<Box<dyn FnOnce()>>,
}

static mut ID: AtomicUsize = AtomicUsize::new(0);
impl Entity {
    fn new() -> Self {
        Self {
            id: unsafe { ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed) },
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
    fn insert(self, id: usize);
    fn drop(id: usize);
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
static mut COLLIDES: OnceCell<SparseSet<Collide>> = OnceCell::new();
impl Component for Collide {
    fn insert(self, id: usize) {
        unsafe {
            COLLIDES.get_or_init(|| SparseSet::with_capacity(2048));
            COLLIDES.get_mut().unwrap().insert(id, self);
        }
    }

    fn drop(id: usize) {
        unsafe {
            COLLIDES.get_mut().unwrap().remove(id);
        }
    }
}

impl ComponentCombination for &Collide {
    fn filter(entity: &Entity) -> Option<Self> {
        unsafe {
            COLLIDES
                .get_or_init(|| SparseSet::with_capacity(2048))
                .get(entity.id)
        }
    }
}
impl ComponentCombination for &mut Collide {
    fn filter(entity: &Entity) -> Option<Self> {
        unsafe {
            COLLIDES.get_or_init(|| SparseSet::with_capacity(2048));
            COLLIDES.get_mut().unwrap().get_mut(entity.id)
        }
    }
}

struct MoveTo {}
impl MoveTo {
    fn move_to(&self) {
        println!("move_to");
    }
}
static mut MOVE_TOS: OnceCell<SparseSet<MoveTo>> = OnceCell::new();
impl Component for MoveTo {
    fn insert(self, id: usize) {
        unsafe {
            MOVE_TOS.get_or_init(|| SparseSet::with_capacity(2048));
            MOVE_TOS.get_mut().unwrap().insert(id, self);
        }
    }

    fn drop(id: usize) {
        unsafe {
            MOVE_TOS.get_mut().unwrap().remove(id);
        }
    }
}
impl ComponentCombination for &MoveTo {
    fn filter(entity: &Entity) -> Option<Self> {
        unsafe {
            MOVE_TOS
                .get_or_init(|| SparseSet::with_capacity(2048))
                .get(entity.id)
        }
    }
}
impl ComponentCombination for &mut MoveTo {
    fn filter(entity: &Entity) -> Option<Self> {
        unsafe {
            MOVE_TOS.get_or_init(|| SparseSet::with_capacity(2048));
            MOVE_TOS.get_mut().unwrap().get_mut(entity.id)
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
