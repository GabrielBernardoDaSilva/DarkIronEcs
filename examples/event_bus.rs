use dark_iron_ecs::core::{event::EventManager, world::World};

pub struct FireEvent(u32);

fn build_up_world(event_bus: &mut EventManager) {
    event_bus.subscribe_event(|_, event: FireEvent| {
        println!("FireEvent: {}", event.0);
    });
}

fn fire_event_system(event_bus: &mut EventManager) {
    event_bus.publish(FireEvent(1));
    std::thread::sleep(std::time::Duration::from_millis(100));
    event_bus.publish(FireEvent(2));
}

fn main() {
    World::default()
        .add_system(
            dark_iron_ecs::core::system::SystemSchedule::Startup,
            build_up_world,
        )
        .add_systems(
            dark_iron_ecs::core::system::SystemSchedule::Update,
            (fire_event_system,),
        )
        .run_startup()
        .run_update();
}
