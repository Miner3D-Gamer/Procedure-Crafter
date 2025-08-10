use crate::{
    internal::{Block, Camera, ID},
    logic::{get_closest_input, Physics},
    SizeType,
};

pub struct LogicFast {}

impl Physics for LogicFast {
    fn new() -> Self {
        LogicFast {}
    }
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
    fn is_rectangle_visible_on_screen<T: mirl::math::Number>(
        &self,
        x: T,
        y: T,
        width: T,
        height: T,
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
        if !self
            .is_point_in_rectangle(x, y, cam_x, cam_y, cam_width, cam_height)
        {
            return false;
        }
        if !self
            .is_point_in_rectangle(x2, y, cam_x, cam_y, cam_width, cam_height)
        {
            return false;
        }
        if !self
            .is_point_in_rectangle(x, y2, cam_x, cam_y, cam_width, cam_height)
        {
            return false;
        }
        if !self
            .is_point_in_rectangle(x2, y2, cam_x, cam_y, cam_width, cam_height)
        {
            return false;
        }
        true
    }
    fn get_block_in_distance(
        &self,
        blocks: &Vec<Block>,
        pos_x: SizeType,
        pos_y: SizeType,
        max_distance: SizeType,
        blacklisted: Option<usize>,
        top: bool,
    ) -> Option<usize> {
        // Any
        for (block_id, block) in blocks.iter().enumerate() {
            if blacklisted.is_some() && block_id == blacklisted.unwrap() {
                continue;
            }
            let check_x;
            let check_y;
            if top {
                check_x = block.x.get() as SizeType;
                check_y = block.y.get() as SizeType;
            } else {
                check_x = block.x.get() as SizeType;
                check_y = block.y.get() as SizeType + block.height.get();
            }
            if block.block_type == 0
                && self.get_distance_between_positions(
                    pos_x, pos_y, check_x, check_y,
                ) < max_distance
            {
                return Some(block_id);
            }
        }
        None
    }
    fn get_distance_between_positions(
        &self,
        x1: SizeType,
        y1: SizeType,
        x2: SizeType,
        y2: SizeType,
    ) -> SizeType {
        let dx = (x1 - x2).abs();
        let dy = (y1 - y2).abs();
        dx.max(dy) + 0.41 * dx.min(dy)
    }
    fn get_block_input_in_distance(
        &self,
        blocks: &Vec<Block>,
        pos_x: SizeType,
        pos_y: SizeType,
        max_distance: SizeType,
        blacklisted: &[ID],
        _top: bool,
    ) -> Option<Vec<(ID, usize)>> {
        let mut smallest = SizeType::MAX;
        let mut smallest_path = Vec::new();
        for block in blocks {
            if blacklisted.contains(&block.id) {
                continue;
            }
            let distances = block.get_inputs_in_range(
                (pos_x, pos_y),
                (0.0, 0.0),
                max_distance,
                self,
                blacklisted,
                blocks,
            );
            if let Some(distance) = distances {
                let (path, dis) = get_closest_input(distance);
                if dis < smallest {
                    smallest_path = path;
                    smallest = dis;
                }
            }
        }
        if smallest_path.is_empty() {
            return None;
        }
        Some(smallest_path)
    }
}
