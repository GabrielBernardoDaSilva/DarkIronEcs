use super::world::World;

pub trait Extension {
    fn build(&self, world: &mut World);
}
