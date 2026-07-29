//! Demonstrates the runtime conflict detection that stops a system from getting two aliasing
//! references to the same data.
//!
//! Each `SystemParam` a system takes (`&mut EntityManager`, `Query<...>`, `Resource<T>`, ...)
//! is fetched via an unsafe raw-pointer dereference, so nothing at the type level stops a
//! system from asking for the same underlying data twice. `Coordinator`'s `AccessTracker`
//! catches that at runtime instead: as soon as two parameters in the same system call would
//! conflict (same manager/component/resource, at least one of them mutable), it panics with a
//! clear message *before* the second aliasing reference is ever created.
//!
//! Run with `cargo run --example system_param_conflicts`.

#![allow(dead_code)]
use dark_iron_ecs::core::{
    entity_manager::EntityManager, query::Query, resources::Resource, system::SystemSchedule,
    world::World,
};

struct Health(i32);
struct Name(String);
struct Camera(f32);

/// Runs `f` in a fresh `World`, printing whether it panicked (conflict caught) or completed.
fn demo(title: &str, expect_conflict: bool, f: impl FnOnce() + std::panic::UnwindSafe) {
    let result = std::panic::catch_unwind(f);
    let panicked = result.is_err();
    let verdict = if panicked {
        "panicked (conflict detected)"
    } else {
        "ran fine"
    };
    let mark = if panicked == expect_conflict {
        "OK"
    } else {
        "UNEXPECTED"
    };
    println!("[{mark}] {title}: {verdict}");
}

fn main() {
    // Silence the panic backtraces printed by `catch_unwind` so the demo output stays readable.
    std::panic::set_hook(Box::new(|_| {}));

    // --- these are all expected to panic ---

    demo("same system takes &mut EntityManager twice", true, || {
        fn conflicting(_a: &mut EntityManager, _b: &mut EntityManager) {}
        World::default()
            .add_system(SystemSchedule::Startup, conflicting)
            .run_startup();
    });

    demo("same system takes Query<&mut Health> twice", true, || {
        fn conflicting(q1: Query<(&mut Health,)>, q2: Query<(&mut Health,)>) {
            let _ = (q1.fetch(), q2.fetch());
        }
        let mut world = World::default();
        world.create_entity((Health(1),));
        world
            .add_system(SystemSchedule::Startup, conflicting)
            .run_startup();
    });

    demo("same system takes Resource<Camera> twice", true, || {
        fn conflicting(_a: Resource<Camera>, _b: Resource<Camera>) {}
        let mut world = World::default();
        world.add_resource(Camera(1.0));
        world
            .add_system(SystemSchedule::Startup, conflicting)
            .run_startup();
    });

    demo(
        "same system takes &mut EntityManager and a Query over it",
        true,
        || {
            // Spawning entities can reallocate the archetype storage the Query is reading from.
            fn conflicting(_entities: &mut EntityManager, q: Query<(&Health,)>) {
                let _ = q.fetch();
            }
            let mut world = World::default();
            world.create_entity((Health(1),));
            world
                .add_system(SystemSchedule::Startup, conflicting)
                .run_startup();
        },
    );

    // --- these are all legitimate and must NOT panic ---

    demo("Query over different components: fine", false, || {
        fn ok(q1: Query<(&Health,)>, q2: Query<(&Name,)>) {
            let _ = (q1.fetch(), q2.fetch());
        }
        let mut world = World::default();
        world.create_entity((Health(1), Name("a".into())));
        world.add_system(SystemSchedule::Startup, ok).run_startup();
    });

    demo(
        "&EntityManager (shared) alongside Query<&mut Health>: fine",
        false,
        || {
            // Reading the entity list while mutating a component's value doesn't restructure
            // anything, so this is safe.
            fn ok(_entities: &EntityManager, q: Query<(&mut Health,)>) {
                let _ = q.fetch();
            }
            let mut world = World::default();
            world.create_entity((Health(1),));
            world.add_system(SystemSchedule::Startup, ok).run_startup();
        },
    );

    demo(
        "two separate systems each taking &mut EntityManager, run sequentially: fine",
        false,
        || {
            fn spawn(entities: &mut EntityManager) {
                entities.create_entity((Health(1),));
            }
            fn clear_all(entities: &mut EntityManager) {
                let _ = entities.entities.len();
            }
            World::default()
                .add_systems(SystemSchedule::Startup, (spawn, clear_all))
                .run_startup();
        },
    );
}
