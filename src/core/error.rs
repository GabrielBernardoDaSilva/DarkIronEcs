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
        write!(f, "ArchetypeError")
    }
}

impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "QueryError")
    }
}


impl Error for ArchetypeError {}
impl Error for QueryError {}
