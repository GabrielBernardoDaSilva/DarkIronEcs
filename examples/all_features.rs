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

fn test_system(q: Query<(&Health,)>, entity_manager: &mut EntityManager) {
    entity_manager.create_entity((Health(500),));
    for health in q.iter() {
        println!("{:?}", health.0);
    }
}

fn test_system_1(
    q: Query<(&Health,)>,
    q2: Query<(&Health,), Without<(&Name,)>>,
    mut camera: Resource<Camera>,
) {
    for health in q.iter() {
        println!("q {:?}", health.0);
    }

    for health in q2.iter() {
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

    world.subscribe_event(|_world: &World, _t: CollisionEvent| {
        println!("Collision Event Hit");
    });

    let entity1 = world.create_entity_with_id((Health(100),));

    let entity2 = world.create_entity_with_id((Name("Enemy 2".to_string()), Health(200)));

    let entity3 = world.create_entity_with_id((
        Position { x: 0.0, y: 0.0 },
        Name("Enemy 3".to_string()),
        Health(300),
    ));

    world.remove_component::<Health>(entity3);

    world.add_component_to_entity(entity3, Health(400));

    let mut counter = 10;
    world.add_coroutine(Coroutine::new("Test Coroutine", move |world| {
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
            return CoroutineState::Finished;
        }

        println!("Coroutine Running");
        CoroutineState::Yielded(WaitAmountOfSeconds {
            amount_in_seconds: 1.0,
        })
    }));

    world.add_resource(Camera {
        x: 1000.0,
        y: 0.0,
        z: 0.0,
    });

    world.add_system(SystemSchedule::Startup, test_system);
    world.add_systems(SystemSchedule::Update, (test_system_1,));

    world.run_update();

    world.publish_event(CollisionEvent);
    world.add_extension(ExtensionExample);


    world.build();
    world.run_startup();
    
}
