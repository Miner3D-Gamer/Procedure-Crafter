use crate::{
    custom::{Block, Camera},
    logic::Physics,
};

pub struct LogicFast {}

impl LogicFast {
    pub fn new() -> Self {
        return LogicFast {};
    }
}

impl Physics for LogicFast {
    fn is_in_any_hole(
        &self,
        x: isize,
        y: isize,
        holes: &[(isize, isize, isize, isize)],
    ) -> bool {
        for &(hx0, hy0, hx1, hy1) in holes {
            if x >= hx0 && x < hx1 && y >= hy0 && y < hy1 {
                return true;
            }
        }
        false
    }
    fn is_reqtuctangle_visible_on_screen(
        &self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        camera: &Camera,
        buffer_width: &isize,
        buffer_height: &isize,
    ) -> bool {
        let cam_x = camera.x as f32;
        let cam_y = camera.y as f32;
        let cam_width = *buffer_width as f32;
        let cam_height = *buffer_height as f32;
        let x2 = x + width;
        let y2 = y + height;
        if !self
            .is_point_in_requctangle(x, y, cam_x, cam_y, cam_width, cam_height)
        {
            return false;
        }
        if !self
            .is_point_in_requctangle(x2, y, cam_x, cam_y, cam_width, cam_height)
        {
            return false;
        }
        if !self
            .is_point_in_requctangle(x, y2, cam_x, cam_y, cam_width, cam_height)
        {
            return false;
        }
        if !self.is_point_in_requctangle(
            x2, y2, cam_x, cam_y, cam_width, cam_height,
        ) {
            return false;
        }
        return true;
    }
    fn get_block_in_distance(
        &self,
        blocks: &Vec<Block>,
        pos_x: f32,
        pos_y: f32,
        max_distance: f32,
        blacklisted: Option<usize>,
        top: bool,
    ) -> Option<usize> {
        // Any
        for block_id in 0..blocks.len() {
            if blacklisted.is_some() {
                if block_id == blacklisted.unwrap() {
                    continue;
                }
            }
            let block = &blocks[block_id];
            let check_x;
            let check_y;
            if top {
                check_x = block.x.get() as f32;
                check_y = block.y.get() as f32;
            } else {
                check_x = block.x.get() as f32;
                check_y = block.y.get() as f32 + block.height.get();
            }
            if block.block_type == 0 {
                if self.get_distance_between_positions(
                    pos_x,
                    pos_y,
                    check_x as f32,
                    check_y as f32,
                ) < max_distance
                {
                    return Some(block_id);
                }
            }
        }
        return None;
    }
    fn get_distance_between_positions(
        &self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    ) -> f32 {
        let dx = (x1 - x2).abs();
        let dy = (y1 - y2).abs();
        dx.max(dy) + 0.41 * dx.min(dy)
    }
}
