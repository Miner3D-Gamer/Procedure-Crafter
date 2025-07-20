use crate::{
    custom::{Block, Camera},
    logic::Physics,
};

pub struct LogicAccurate {}

impl LogicAccurate {
    pub fn new() -> Self {
        LogicAccurate {}
    }
}

impl Physics for LogicAccurate {
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
    fn is_rectangle_visible_on_screen<F: mirl::math::Number>(
        &self,
        x: F,
        y: F,
        width: F,
        height: F,
        camera: &Camera,
        buffer_width: &isize,
        buffer_height: &isize,
    ) -> bool {
        let x = x.to_f64().unwrap();
        let y = y.to_f64().unwrap();
        let cam_x = camera.x as f64;
        let cam_y = camera.y as f64;
        let cam_width = *buffer_width as f64;
        let cam_height = *buffer_height as f64;
        let x2 = x + (width).to_f64().unwrap();
        let y2 = y + (height).to_f64().unwrap();
        if self
            .is_point_in_rectangle(x, y, cam_x, cam_y, cam_width, cam_height)
        {
            return true;
        }
        if self
            .is_point_in_rectangle(x2, y, cam_x, cam_y, cam_width, cam_height)
        {
            return true;
        }
        if self
            .is_point_in_rectangle(x, y2, cam_x, cam_y, cam_width, cam_height)
        {
            return true;
        }
        if self.is_point_in_rectangle(
            x2, y2, cam_x, cam_y, cam_width, cam_height,
        ) {
            return true;
        }
        false
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
        let mut closest = None;
        let mut min_distance = max_distance; // Start with max distance as the limit

        for (block_id, block) in blocks.iter().enumerate() {
            if blacklisted.is_some() {
                if block_id == blacklisted.unwrap() {
                    continue;
                }
            }
            let check_x;
            let check_y;
            if top {
                check_x = block.x.get() as f32;
                check_y = block.y.get() as f32;
            } else {
                check_x = block.x.get() as f32;
                check_y = block.y.get() as f32 + block.height.get();
            }
            let distance = self.get_distance_between_positions(
                pos_x,
                pos_y,
                check_x as f32,
                check_y as f32,
            );

            if distance < min_distance {
                min_distance = distance;
                closest = Some(block_id); // Problem may be here
            }
        }

        closest
    }
    fn get_distance_between_positions(
        &self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    ) -> f32 {
        ((x1 - x2) * (x1 - x2) + (y1 - y2) * (y1 - y2)).sqrt()
    }
    fn get_block_input_in_distance(
        &self,
        blocks: &Vec<Block>,
        pos_x: f32,
        pos_y: f32,
        max_distance: f32,
        blacklisted: Option<usize>,
        top: bool,
    ) -> Option<usize> {
        None
    }
}
