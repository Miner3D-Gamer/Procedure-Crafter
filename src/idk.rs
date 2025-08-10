use fontdue::Font;

use crate::{
    internal::{Block, Camera},
    logic::Physics,
};
use mirl::{
    graphics::adjust_brightness_fast,
    platform::Buffer,
    render::{draw_rectangle, draw_text_antialiased},
};

#[inline]
pub fn draw_block<L: Physics>(
    block: &Block,
    origin_x: isize,
    origin_y: isize,
    camera: &Camera,
    buffer: &Buffer,
    block_color: u32,
    _width: usize,
    _height: usize,
    font: &Font,
    _logic: &L,
) {
    let origin_x = origin_x - camera.x;
    let origin_y = origin_y - camera.y;

    draw_rectangle(
        buffer,
        origin_x,
        origin_y,
        block.width.get() as isize,
        block.height.get() as isize,
        block_color,
        true,
    );
    let length = block.name.len();
    let last_index;
    if length > 0 {
        last_index = length - 1;
        let indexes = 0..last_index;

        for i in indexes.clone() {
            draw_text_antialiased(
                buffer,
                &block.name[i],
                origin_x as usize
                    + block.input_offsets.borrow()[i * 2] as usize,
                origin_y as usize,
                mirl::graphics::rgb_to_u32(255, 0, 0),
                (block.height.get() / 2.0) as f32,
                font,
                true,
            );
            //framework.log("{}", block.height.get());
            // framework.log(
            //     "{:#?} for {:#?} expected {:#?}",
            //     block.input_offsets.borrow(),
            //     block.internal_name,
            //     indexes
            // );

            draw_rectangle(
                buffer,
                origin_x + block.input_offsets.borrow()[i * 2 + 1] as isize,
                origin_y + block.height.get() as isize / 10,
                block.input_offsets.borrow()[i * 2 + 2] as isize
                    - block.input_offsets.borrow()[i * 2 + 1] as isize,
                block.height.get() as isize - block.height.get() as isize / 5,
                adjust_brightness_fast(block_color, 50),
                true,
            );
        }
    } else {
        last_index = 0;
    }
    let l = block.input_offsets.borrow().len();
    if l > 0 {
        draw_text_antialiased(
            buffer,
            &block.name[last_index],
            (origin_x - camera.x) as usize
                + block.input_offsets.borrow()[l - 1] as usize,
            (origin_y - camera.y) as usize,
            mirl::graphics::rgb_to_u32(255, 0, 0),
            20.0,
            font,
            true,
        );
    }

    // self.draw_text(
    //     buffer,
    //     width,
    //     height,
    //     &block.name[last_index],
    //     (origin_x - camera.x) as usize + offset as usize,
    //     (origin_y - camera.y) as usize,
    //     mirl::graphics::rgb_to_u32(255, 0, 0),
    //     20.0,
    //     font,
    // );
}
