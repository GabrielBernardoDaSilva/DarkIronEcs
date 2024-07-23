#![allow(dead_code)]

use prometheus_ecs::{
    entity_manager::EntityManager, event::EventManager, query::{Query, Without}, resources::Resource, system::{IntoSystem, System, SystemManager}, world::World
};

pub mod prometheus_ecs;

#[derive(Debug)]
struct Position {
    x: f32,
    y: f32,
}
#[derive(Debug)]
struct Name(String);
#[derive(Debug)]
struct Health(i32);

fn test_system(q: Query<(&Health,)>, entity_manager: &mut EntityManager) {
    println!("{:?}", q.components);
    entity_manager.create_entity((Health(500), ));
    println!("{:?}", entity_manager.archetypes);
    for health in q.iter() {
        println!("{:?}", health);
    }

    
}

fn test_system_1(
    q: Query<(&Health,)>,
    q2: Query<(&Health,), Without<(&Name,)>>,
    entity_manager: &EntityManager,
    camera: Resource<Camera>
) {
    println!("{:?}", q.components.iter());
    println!("{:?}", q2.components.iter());
    println!("{:?}", entity_manager.archetypes);
    println!("Hello from test_system_1");
    println!("{:?} {:?} {:?}", camera.x, camera.y, camera.z);
}

struct CollisionEvent;

struct Camera{
    x: f32,
    y: f32,
    z: f32,
}

fn main() {
    let mut world = prometheus_ecs::world::World::new();

    world.subscribe_event(|_world: &World, _t: CollisionEvent| {
        println!("Collision Event Hit");
    });

    world.create_entity((Health(100), ));

    world.create_entity((Name("Enemy 2".to_string()), Health(200)));

    world.create_entity((
        Position { x: 0.0, y: 0.0 },
        Name("Enemy 3".to_string()),
        Health(300),
    ));

    world.add_resource(Camera { x: 1000.0, y: 0.0, z: 0.0 });


    // world.create_entity((Position { x: 2.0, y: 2.0 }, Health(400)));

    world.add_system(test_system);

    world.add_system(test_system_1);

    world.run_systems();

    world.publish_event(CollisionEvent);

    // world.run_systems();
}
