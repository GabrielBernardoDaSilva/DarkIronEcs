use prometheus_ecs::{
    entity_manager::EntityManager, query::Query, system::SystemSchedule, world::World,
};

pub mod prometheus_ecs;

struct Position {
    x: f32,
    y: f32,
}

struct Name(String);

fn add_pawns(entity_manager: &mut EntityManager) {
    for i in 0..1000000 {
        entity_manager.create_entity((
            Position {
                x: i as f32,
                y: i as f32,
            },
            Name(format!("Pawn {}", i)),
        ));
    }

    println!("All pawns initialized");
}

fn move_pawns(q: Query<(&mut Position,)>) {
    println!("Moving pawns");
    for position in q.iter() {
        position.x += 1.0;
        position.y += 1.0;
    }
}

fn print_pawns(q: Query<(&Position, &Name)>) {
    for (position, name) in q.iter() {
        println!("Name: {}", name.0);
        println!("Position: x: {}, y: {}", position.x, position.y);
    }
}

fn main() {
    let start = std::time::Instant::now();
    let world = World::new();
    world.add_systems(
        SystemSchedule::Startup,
        (add_pawns, move_pawns, print_pawns),
    );
    world.run_startup();

    let duration = start.elapsed();
    println!("Time elapsed in building world: {:?}", duration);
}


// Time elapsed in building world: 121.1069256s