# Dark Iron ECS

<p align="center">
      <img src='https://raw.githubusercontent.com/GabrielBernardoDaSilva/DarkIronEcs/master/logo/darkiron.png' alt='Dark Iron'/>
</p>

A lightweight and flexible Entity Component System (ECS) library for Rust.

## Features

- **Entity Creation**: Easily create entities and attach components to them.
- **Component Queries**: Efficiently query entities based on their components, with support for exclusion constraints via `Without<T>`.
- **Systems**: Implement game logic through systems that run on `Startup`, `Update`, or `Shutdown` schedules.
- **Events**: Event-driven architecture with custom events and typed handlers.
- **Coroutines**: Coroutines executed at specific intervals or delays, with named lifecycle management.
- **Resources**: Globally accessible shared data, injectable directly into systems.
- **Extensions**: Extend the world setup with reusable `Extension` implementations.
- **Fluent Builder API**: World can be configured through a chain of methods.

## Setup

Add `dark_iron_ecs` to your `Cargo.toml`:

```toml
[dependencies]
dark_iron_ecs = "0.9.5"
```

## Quick Example

```rust
use dark_iron_ecs::core::{
    world::World,
    system::SystemSchedule,
    query::Query,
};

#[derive(Debug)]
struct Health(i32);

#[derive(Debug)]
struct Position { x: f32, y: f32 }

fn health_system(q: Query<(&Health,)>) {
    for (health,) in q.fetch() {
        println!("Health: {:?}", health.0);
    }
}

fn main() {
    let mut world = World::new();

    world
        .create_entity((Health(100), Position { x: 0.0, y: 0.0 }))
        .create_entity((Health(200),))
        .add_system(SystemSchedule::Update, health_system)
        .run_startup();

    world.run_update();
}
```

## Components

Any type can be used as a component — no derive or trait implementation required:

```rust
struct Position { x: f32, y: f32 }
struct Name(String);
struct Health(i32);
```

## Systems

Systems are plain functions. Parameters are injected automatically — queries, resources, and managers are all valid system parameters:

```rust
use dark_iron_ecs::core::{
    query::{Query, Without},
    resources::Resource,
    entity_manager::EntityManager,
};

fn movement_system(q: Query<(&mut Position, &Health)>) {
    for (pos, health) in q.fetch() {
        pos.x += 1.0;
        println!("Health: {:?}", health.0);
    }
}

fn spawn_system(entity_manager: &mut EntityManager) {
    entity_manager.create_entity((Health(500),));
}

fn filtered_system(q: Query<(&Health,), Without<(&Name,)>>) {
    for (health,) in q.fetch() {
        println!("Entity without Name — Health: {:?}", health.0);
    }
}
```

## Events

```rust
struct CollisionEvent;

fn setup(world: &mut World) {
    world.subscribe_event(|_world: &World, _event: CollisionEvent| {
        println!("Collision detected!");
    });

    // Later, publish the event:
    world.publish_event(CollisionEvent);
}
```

## Resources

```rust
struct Camera { x: f32, y: f32, z: f32 }

fn camera_system(mut camera: Resource<Camera>) {
    camera.x += 1.0;
    println!("Camera: {} {} {}", camera.x, camera.y, camera.z);
}

fn setup(world: &mut World) {
    world.add_resource(Camera { x: 0.0, y: 0.0, z: 0.0 });
}
```

## Coroutines

Coroutines are named and support yielding execution for a duration:

```rust
use dark_iron_ecs::core::coroutine::{Coroutine, CoroutineState, WaitAmountOfSeconds};

let coroutine = Coroutine::new("my-coroutine", move |world| {
    println!("Coroutine tick");
    CoroutineState::Yielded(WaitAmountOfSeconds { amount_in_seconds: 1.0 })
    // Return CoroutineState::Finished to stop
});

world.add_coroutine(coroutine);

// Call every frame with delta time:
world.update_coroutines(delta_time);
```

> Coroutine names must be unique — adding a duplicate name will panic.

## Extensions

Extensions are reusable world setup blocks:

```rust
use dark_iron_ecs::core::extension::Extension;

pub struct PhysicsExtension;

impl Extension for PhysicsExtension {
    fn build(&self, world: &mut World) {
        world.create_entity((Health(100),));
        // register systems, resources, etc.
    }
}

world.add_extension(PhysicsExtension).build();
```

## Entity Component Access

Entities returned by `create_entity_with_id` can be used to access components directly:

```rust
let entity = world.create_entity_with_id((Health(100),));

if let Some(health) = entity.get_component::<Health>(&world) {
    println!("Health: {:?}", health.0);
}
```

## System Schedules

| Schedule                   | When it runs                            |
| -------------------------- | --------------------------------------- |
| `SystemSchedule::Startup`  | Once, via `world.run_startup()`         |
| `SystemSchedule::Update`   | Every frame, via `world.run_update()`   |
| `SystemSchedule::Shutdown` | On shutdown, via `world.run_shutdown()` |

## Running the Examples

```bash
cargo run --example simple_query
cargo run --example all_features
cargo run --example benchmark
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

MIT License

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
