# Dark Iron ECS

<p align="center">
      <img src='https://raw.githubusercontent.com/GabrielBernardoDaSilva/DarkIronEcs/master/logo/darkiron.png' alt='Dark Iron'/>
</p>



## Features

- **Entity Creation**: Easily create entities and attach components to them.
- **Event Handling**: Implement event-driven architecture with custom events and event handlers.
- **Coroutines**: Create a coroutine to be executed at specific intervals or delays.
- **Queries**: Efficiently query entities based on their components.
- **Systems**: Implement game logic and behaviors through systems that process entities based on queries.
- **Extensions**: Extend the functionality of the ECS with custom extensions.
- **Chained Building**: World could be create by a chain of methods.

## Example 
Demonstrates how to use the `dark_iron_ecs` library to create an Entity Component System (ECS) in Rust. The example includes creating components, querying entities, and setting up systems to interact with those components.

## Setup

First, add the `dark_iron_ecs` dependency to your `Cargo.toml`:

```toml
[dependencies]
dark_iron_ecs = "0.5.0"  # Replace with the actual version
```

### Imports
For this example the necessary imports: 

```rust
use dark_iron_ecs::core::{
    coroutine::{Coroutine, CoroutineState, WaitAmountOfSeconds},
    entity_manager::EntityManager,
    extension::Extension,
    query::{Query, Without},
    resources::Resource,
    system::SystemSchedule,
    world::World,
};

```

### Components
Define your components. Components are data associated with entities.

```rust
struct Position {
    x: f32,
    y: f32,
}

struct Name(String);

struct Health(i32);
```

### Systems
Define systems to operate on entities that have specific components. Systems are functions that process entities.

```rust
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
```

### Events

Define events that can be published and subscribed to within the world.
```rust
struct CollisionEvent;
```

### Resources

Define resources that are globally accessible by systems.
```rust
struct Camera {
    x: f32,
    y: f32,
    z: f32,
}
```

### Extension

Define easy extensions to your world.

```rust
pub struct ExtensionExample;
impl Extension for ExtensionExample {
    fn build(&self, world: &mut World) {
        world.create_entity((Health(100),));
    }
}
```

### Main Function

Set up the world, create entities, add systems, and run the ECS.

```rust
use dark_iron_ecs::core::{
    coroutine::{Coroutine, CoroutineState, WaitAmountOfSeconds},
    entity_manager::EntityManager,
    query::{Query, Without},
    resources::Resource,
    system::SystemSchedule,
    world::World,
};

fn main() {
    let mut world = World::new();

    let entity1 = world.create_entity_with_id((Health(100),));

    let entity2 = world.create_entity_with_id((Name("Enemy 2".to_string()), Health(200)));

    let entity3 = world.create_entity_with_id((
        Position { x: 0.0, y: 0.0 },
        Name("Enemy 3".to_string()),
        Health(300),
    ));

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
```

### Running the Project
To run the project, use Cargo:

```bash
cargo run
```

This will compile and run your ECS example, demonstrating how to create and manage entities, components, systems, events, and resources using the dark_iron_ecs library.

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
