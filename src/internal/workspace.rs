use crate::logic::Physics;

use super::{Block, Camera};

pub struct WorkSpace<'a, L: Physics + Sized> {
    pub logic: &'a L,
    block_counter: usize,
    pub blocks: Vec<Block>,
    pub camera: Camera,
}

impl<'a, L: Physics> WorkSpace<'a, L> {
    pub fn new(logic: &'a L) -> Self {
        Self {
            logic,
            block_counter: 0,
            blocks: Vec::new(),
            camera: Camera::new(),
        }
    }
    pub fn increment_block_id(&mut self) -> usize {
        self.block_counter += 1;
        self.block_counter
    }
}
