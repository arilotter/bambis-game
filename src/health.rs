use crate::prelude::*;

#[derive(Component, Default, Reflect)]
pub struct Health {
    pub hp: usize,
    pub max: usize,
}

impl Health {
    pub fn new(max: usize) -> Self {
        Self { hp: max, max }
    }
}
