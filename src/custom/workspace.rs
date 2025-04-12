use crate::{logic::Physics, render::RenderSettings};

use super::{Block, Camera};

pub struct WorkSpace<'a, S: RenderSettings, L: Physics> {
    pub render: &'a S,
    pub logic: &'a L,
    block_counter: usize,
    pub action_blocks: Vec<Block>,
    pub inline_blocks: Vec<Block>,
    pub camera: Camera,
}

impl<'a, S: RenderSettings, L: Physics> WorkSpace<'a, S, L> {
    pub fn new(render: &'a S, logic: &'a L) -> Self {
        Self {
            render: render,
            logic: logic,
            block_counter: 0,
            action_blocks: Vec::new(),
            inline_blocks: Vec::new(),
            camera: Camera::new(),
        }
    }
    pub fn increment_block_id(&mut self) -> usize {
        self.block_counter += 1;
        return self.block_counter;
    }
}
