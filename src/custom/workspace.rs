use crate::logic::Physics;

use super::{Block, Camera};

pub struct WorkSpace<'a, L: Physics> {
    pub logic: &'a L,
    block_counter: usize,
    pub action_blocks: Vec<Block>,
    pub inline_blocks: Vec<Block>,
    pub camera: Camera,
}

impl<'a, L: Physics> WorkSpace<'a, L> {
    pub fn new(logic: &'a L) -> Self {
        Self {
            logic: logic,
            block_counter: 0,
            action_blocks: Vec::new(),
            inline_blocks: Vec::new(),
            camera: Camera::new(),
        }
    }
    pub fn increment_block_id(&mut self) -> usize {
        self.block_counter += 1;
        self.block_counter
    }
}
