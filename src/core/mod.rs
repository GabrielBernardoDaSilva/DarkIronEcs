//! Core ECS types. Start at [`world::World`], the main entry point for building and driving an
//! application.

/// Runtime detection of conflicting `SystemParam` accesses within a single system call.
pub(crate) mod access;
/// Archetype-based component storage, grouping entities by their exact component-type set.
pub mod archetype;
/// Downcasting helper used to store heterogeneous, type-erased values (events, resources).
pub mod as_any_trait;
/// Component and component-bundle traits.
pub mod component;
/// [`coordinator::Coordinator`], the handle systems use to reach every manager.
pub mod coordinator;
/// Frame-driven, yieldable [`coroutine::Coroutine`] tasks.
pub mod coroutine;
/// The [`entity::Entity`] handle type.
pub mod entity;
/// Owns entities and their components.
pub mod entity_manager;
/// Error types returned by archetype/query/entity lookups.
pub mod error;
/// Publish/subscribe event system.
pub mod event;
/// Reusable [`extension::Extension`] setup bundles.
pub mod extension;
/// Component queries, with optional [`query::Without`] filtering.
pub mod query;
/// Global, type-keyed resources.
pub mod resources;
/// System registration and scheduling.
pub mod system;
/// [`world::World`], the ECS entry point.
pub mod world;
