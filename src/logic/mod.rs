pub trait Physics {
    fn is_in_any_hole(
        &self,
        x: isize,
        y: isize,
        holes: &[(isize, isize, isize, isize)],
    ) -> bool;
    fn is_rectangle_visible_on_screen<T: mirl::math::Number>(
        &self,
        x: T,
        y: T,
        width: T,
        height: T,
        camera: &Camera,
        buffer_width: &isize,
        buffer_height: &isize,
    ) -> bool;
    fn is_point_in_rectangle<T: mirl::math::Number>(
        &self,
        x: T,
        y: T,
        origin_x: T,
        origin_y: T,
        width: T,
        height: T,
    ) -> bool {
        if x < origin_x {
            return false;
        }
        if x > origin_x + width {
            return false;
        }
        if y < origin_y {
            return false;
        }
        if y > origin_y + height {
            return false;
        }
        true
    }
    fn get_block_in_distance(
        &self,
        blocks: &Vec<Block>,
        pos_x: f32,
        pos_y: f32,
        max_distance: f32,
        blacklisted: Option<usize>,
        top: bool,
    ) -> Option<usize>;
    fn get_distance_between_positions(
        &self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    ) -> f32;
    fn is_block_visible_on_screen(
        &self,
        block: &Block,
        camera: &Camera,
        width: &isize,
        height: &isize,
    ) -> bool {
        self.is_rectangle_visible_on_screen(
            block.x.get() as f32,
            block.y.get() as f32,
            block.width.get(),
            block.height.get(),
            camera,
            width,
            height,
        )
    }
    fn get_block_input_in_distance(
        &self,
        blocks: &Vec<Block>,
        pos_x: f32,
        pos_y: f32,
        max_distance: f32,
        blacklisted: &[ID],
        top: bool,
    ) -> Option<Vec<(ID, usize)>>;
}

mod fast;
pub use fast::LogicFast;

mod accurate;
pub use accurate::LogicAccurate;

use crate::custom::{Block, Camera, ID};

const _: fn() = || {
    fn assert_impl<T: Physics>() {}
    assert_impl::<LogicFast>();
    assert_impl::<LogicAccurate>();
};
