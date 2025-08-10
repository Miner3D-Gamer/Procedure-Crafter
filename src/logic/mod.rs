pub trait Physics {
    fn new() -> Self
    where
        Self: Sized;
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
    fn is_point_in_rectangle<
        T: Copy + PartialOrd + std::ops::Add<Output = T>,
    >(
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
        pos_x: SizeType,
        pos_y: SizeType,
        max_distance: SizeType,
        blacklisted: Option<usize>,
        top: bool,
    ) -> Option<usize>;
    fn get_distance_between_positions(
        &self,
        x1: SizeType,
        y1: SizeType,
        x2: SizeType,
        y2: SizeType,
    ) -> SizeType;
    fn is_block_visible_on_screen(
        &self,
        block: &Block,
        camera: &Camera,
        width: &isize,
        height: &isize,
    ) -> bool {
        self.is_rectangle_visible_on_screen(
            block.x.get() as SizeType,
            block.y.get() as SizeType,
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
        pos_x: SizeType,
        pos_y: SizeType,
        max_distance: SizeType,
        blacklisted: &[ID],
        top: bool,
    ) -> Option<Vec<(ID, usize)>>;
}

mod fast;
pub use fast::LogicFast;

mod accurate;
pub use accurate::LogicAccurate;

use crate::{
    internal::{block::InputRememberer, Block, Camera, ID},
    SizeType,
};

const _: fn() = || {
    fn assert_impl<T: Physics>() {}
    assert_impl::<LogicFast>();
    assert_impl::<LogicAccurate>();
};

pub fn get_closest_input(
    inputs: InputRememberer,
) -> (Vec<(ID, usize)>, SizeType) {
    let mut smallest_path = Vec::new();
    let mut smallest = SizeType::MAX;

    for (spot, more, distance) in inputs.internal_inputs {
        let mut current_path = vec![(inputs.own_id, spot)];
        let distance = distance.unwrap();

        if let Some(deeper) = more {
            let (deeper_path, deeper_distance) = get_closest_input(deeper);
            if deeper_distance < smallest {
                current_path.extend(deeper_path);
                smallest_path = current_path;
                smallest = deeper_distance;
            }
        } else if distance < smallest {
            smallest = distance;
            smallest_path = current_path;
        }
    }

    (smallest_path, smallest)
}
