use crate::render::RenderSettings;

pub struct WorkSpace<'a, S: RenderSettings> {
    render: &'a S,
    block_counter: usize,
}

impl<'a, S: RenderSettings> WorkSpace<'a, S> {
    pub fn new(render: &'a S) -> Self {
        Self {
            render,
            block_counter: 0,
        }
    }
    pub fn increment_block_id(&mut self) -> usize {
        self.block_counter += 1;
        return self.block_counter;
    }
}
