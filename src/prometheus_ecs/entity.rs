pub type EntityId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
    pub id: EntityId,
    pub entity_location: usize,
}

impl Entity {
    pub(crate) fn new(id: EntityId, entity_location: usize) -> Self {
        Entity { id, entity_location }
    }
}
