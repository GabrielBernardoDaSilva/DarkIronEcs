#![allow(unused_imports)]
#![allow(dead_code)]

use dark_iron_ecs::core::{
    coroutine::{Coroutine, CoroutineState, WaitAmountOfSeconds},
    entity_manager::EntityManager,
    extension::Extension,
    query::{Query, Without},
    resources::Resource,
    system::SystemSchedule,
    world::World,
};

struct Position {
    x: f32,
    y: f32,
}

struct Name(String);

struct Health(i32);

struct Velocity(f32, f32);

fn test_system(q: Query<(&Health,)>, entity_manager: &mut EntityManager) {
    entity_manager.create_entity((Health(500),));
    for health in q.fetch() {
        println!("{:?}", health.0);
    }
}

fn test_system_1(
    q: Query<(&Health,)>,
    q2: Query<(&Health,), Without<(&Name,)>>,
    mut camera: Resource<Camera>,
) {
    for health in q.fetch() {
        println!("q {:?}", health.0);
    }

    for health in q2.fetch() {
        println!("q2 {:?}", health.0);
    }

    println!("Hello from test_system_1");
    camera.x += 1.0;
    println!("{:?} {:?} {:?}", camera.x, camera.y, camera.z);
}

struct CollisionEvent;

struct Camera {
    x: f32,
    y: f32,
    z: f32,
}

pub struct ExtensionExample;
impl Extension for ExtensionExample {
    fn build(&self, world: &mut World) {
        world.create_entity((Health(100),));
    }
}

fn main() {
    let mut world = World::new();

    let entity1 = world.create_entity_with_id((Health(100),));

    let entity2 = world.create_entity_with_id((Name("Enemy 2".to_string()), Health(200)));

    let entity3 = world.create_entity_with_id((
        Position { x: 0.0, y: 0.0 },
        Name("Enemy 3".to_string()),
        Health(300),
    ));

    if let Some(health) = entity1.get_component::<Health>(&world){
        println!("Component {:?}", health.0);
    }

    let mut counter = 10;
    world
        .create_entity((Velocity(0.0, 0.0),))
        .subscribe_event(|_world: &World, _t: CollisionEvent| {
            println!("Collision Event Hit");
        })
        .remove_component::<Health>(entity3)
        .add_component_to_entity(entity3, Health(400))
        .add_coroutine(Coroutine::new("Test Coroutine", move |world| {
            if counter == 10 {
                println!("Coroutine Started");
            }

            counter -= 1;
            if counter == 0 {
                println!("Coroutine Finished");
                world.remove_entity(entity3);
                world.create_entity((Health(900),));
                world.add_component_to_entity(entity1, Name("Player".to_string()));
                world.remove_component::<Health>(entity2);
                world.publish_event(CollisionEvent);
                return CoroutineState::Finished;
            }

            println!("Coroutine Running");
            CoroutineState::Yielded(WaitAmountOfSeconds {
                amount_in_seconds: 1.0,
            })
        }))
        .add_resource(Camera {
            x: 1000.0,
            y: 0.0,
            z: 0.0,
        })
        .add_system(SystemSchedule::Startup, test_system)
        .add_systems(SystemSchedule::Update, (test_system_1,))
        .run_startup()
        .add_extension(ExtensionExample)
        .build();

    let q = world.create_query::<&Health>();
    let q1 = world.create_query_with_constraint::<&Health, Without<&Name>>();

    for health in q.fetch() {
        println!("{:?}", health.0);
    }

    for health in q1.fetch() {
        println!("{:?}", health.0);
    }

    loop {
        world.run_update();
        world.update_coroutines(1.0);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
