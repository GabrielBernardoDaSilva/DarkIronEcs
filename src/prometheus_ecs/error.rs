
use std::fmt::Debug;

pub enum ArchetypeError {
    EntityNotFound,
}


impl Debug for ArchetypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ArchetypeError")
    }
}
