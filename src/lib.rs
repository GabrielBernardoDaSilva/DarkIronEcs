//! A simple, single-threaded Entity Component System.
//!
//! ```
//! use dark_iron_ecs::core::world::World;
//!
//! let mut world = World::default();
//! world.run_startup();
//! world.run_update();
//! ```
//!
//! See [`core::world::World`] for the main entry point, and the crate README for a tour of
//! entities/components, queries, systems, events, resources, coroutines and extensions.
pub mod core;
