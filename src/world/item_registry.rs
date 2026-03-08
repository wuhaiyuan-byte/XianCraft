use crate::world::item::ItemPrototype;
use std::collections::HashMap;

#[derive(Debug)]
pub struct ItemRegistry {
    prototypes: HashMap<u32, ItemPrototype>,
}

impl ItemRegistry {
    pub fn new(prototypes: HashMap<u32, ItemPrototype>) -> Self {
        Self { prototypes }
    }

    pub fn get_prototype(&self, id: &u32) -> Option<&ItemPrototype> {
        self.prototypes.get(id)
    }
}
