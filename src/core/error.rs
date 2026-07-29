use std::{error::Error, fmt::Debug};

use super::entity::EntityId;

/// Errors returned by [`Archetype`](super::archetype::Archetype) operations.
#[derive(Debug)]
pub enum ArchetypeError {
    /// The entity isn't present in the archetype.
    EntityNotFound,
}

/// Errors returned when looking up an entity's component, e.g. via
/// [`Entity::get_component`](super::entity::Entity::get_component) or a [`Query`](super::query::Query).
#[derive(Debug)]
pub enum QueryError {
    /// No entity with this id exists.
    EntityNotFound(EntityId),
    /// The entity exists but doesn't have a component of this type.
    ComponentNotFound(String)
}



impl std::fmt::Display for ArchetypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArchetypeError::EntityNotFound => write!(f, "ArchetypeError: entity not found"),
        }
    }
}

impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryError::EntityNotFound(id) => write!(f, "QueryError: entity {} not found", id),
            QueryError::ComponentNotFound(name) => write!(f, "QueryError: component '{}' not found", name),
        }
    }
}


impl Error for ArchetypeError {}
impl Error for QueryError {}
