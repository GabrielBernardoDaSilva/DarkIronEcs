use std::{error::Error, fmt::Debug};

use super::entity::EntityId;

#[derive(Debug)]
pub enum ArchetypeError {
    EntityNotFound,
}

#[derive(Debug)]
pub enum QueryError {
    EntityNotFound(EntityId),
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
