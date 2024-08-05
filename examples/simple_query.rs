#![allow(unused_imports)]
#![allow(dead_code)]

use dark_iron_ecs::core::{query::Query, system::SystemSchedule, world::World};

struct Position {
    x: f32,
    y: f32,
}

struct Name(String);

pub struct Health(i32);

fn get_player_health(q: Query<(&Position, &Health)>) {
    for (_, health) in q.fetch() {
        println!("Player health: {:?}", health.0);
    }
}

fn main() {
    let mut world = World::new();

    world
        .create_entity((
            Position { x: 0.0, y: 0.0 },
            Name("Player".to_string()),
            Health(100),
        ))
        .create_entity((Position { x: 1.0, y: 1.0 }, Name("Enemy".to_string())))
        .add_system(SystemSchedule::Update, get_player_health)
        .run_startup();

    loop {
        world.run_update();
    }
}
