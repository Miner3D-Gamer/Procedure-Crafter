#![allow(dead_code)]
#![allow(static_mut_refs)] // Yeah, this is probably fine

// Bugs:
// When a block structure is added to a another block structure, subsequent blocks of the moved blocks don't get tagged with 'recently_moved'
// There is a line where the code keeps on erroring ->

use core::panic;

use fontdue::Font;
use mirl::graphics::rgb_to_u32;

use std::cell::Cell;

// Cell: Make attribute mutable while keeping the rest of the struct static

use crate::custom_id::ID;
use crate::platform::shared::FileSystem;

struct Camera {
    x: isize,
    y: isize,
    z: f32,
}

struct Block {
    name: String,
    internal_name: String,
    x: Cell<u16>,
    y: Cell<u16>,
    width: Cell<f32>,
    height: Cell<f32>,
    block_type: u8,
    // 0: Action, 1: Inline, 2: Hugging, 3: Event
    required_imports: Vec<String>,
    required_contexts: Vec<String>,
    file_versions: Vec<String>,
    file_locations: Vec<String>,
    output: String,
    inputs: Vec<BlockInput>,
    block_color_id: usize,
    id: ID,
    connected_top: Cell<Option<ID>>,
    connected_below: Cell<Option<ID>>,
    possible_connection_above: Cell<Option<ID>>,
    possible_connection_below: Cell<Option<ID>>,
    recently_moved: Cell<bool>,
}

impl Block {
    fn new(
        name: String,
        internal_name: String,
        x: i16,
        y: i16,
        block_type: u8,
        required_imports: Vec<String>,
        required_contexts: Vec<String>,
        file_versions: Vec<String>,
        file_locations: Vec<String>,
        output: String,
        inputs: Vec<BlockInput>,
        outuput_color_names: &Vec<String>,
        font: &Font,
    ) -> Block {
        let color_id = outuput_color_names
            .iter()
            .position(|x| *x == output)
            .expect("Could not find color name");
        let x = ((u16::MAX / 2) as i16 + x) as u16;
        let y = ((u16::MAX / 2) as i16 + y) as u16;

        Block {
            name: name.clone(),
            internal_name: internal_name,
            x: Cell::new(x),
            y: Cell::new(y),
            width: Cell::new(get_length_of_text_in_font(&name, &font)),
            height: Cell::new(40.0),
            block_type: block_type,
            required_imports: required_imports,
            required_contexts: required_contexts,
            file_versions: file_versions,
            file_locations: file_locations,
            output: output,
            inputs: inputs,
            block_color_id: color_id,
            id: increment_global_block_id().into(),
            connected_top: Cell::new(None),
            connected_below: Cell::new(None),
            possible_connection_above: Cell::new(None),
            possible_connection_below: Cell::new(None),
            recently_moved: Cell::new(false),
        }
    }
}

struct BlockInput {
    input_type: String,
    name: String,
    expected: Option<Vec<String>>,
    expected_return: Option<Vec<String>>,
}
impl BlockInput {
    fn new(
        input_type: String,
        name: String,
        expected: Option<Vec<String>>,
        expected_return: Option<Vec<String>>,
    ) -> Result<Self, &'static str> {
        if let (Some(ref e), Some(ref er)) = (&expected, &expected_return) {
            if e.len() != er.len() {
                return Err(
                    "expected and expected_return must have the same length",
                );
            }
        }

        Ok(Self {
            input_type,
            name,
            expected,
            expected_return,
        })
    }
}

use fontdue::Metrics;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::Once;

static INIT: Once = Once::new();
static mut GLYPH_CACHE: Option<
    RefCell<HashMap<(char, (i32, i32)), (Metrics, Vec<u8>)>>,
> = None;

#[inline(always)]
fn get_glyph_cache(
) -> &'static RefCell<HashMap<(char, (i32, i32)), (Metrics, Vec<u8>)>> {
    unsafe {
        INIT.call_once(|| {
            GLYPH_CACHE = Some(RefCell::new(HashMap::new()));
        });
        GLYPH_CACHE.as_ref().unwrap()
    }
}
#[inline(always)]
fn round_float_key(value: f32) -> (i32, i32) {
    let multiplier = 10000.0; // Provides 4 decimal places of precision
    let rounded_int_x = (value * multiplier).round() as i32;
    let rounded_int_y = (value * multiplier).fract() as i32;
    (rounded_int_x, rounded_int_y)
}

// Drawing
fn draw_text(
    buffer: *mut u32,
    width: &usize,
    height: &usize,
    text: &str,
    x: usize,
    y: usize,
    color: u32,
    size: f32,
    font: &Font,
) {
    // This function takes like half the render time
    if FAST_RENDER {
        draw_text_no_blend(
            buffer, width, height, text, x, y, color, size, font,
        );
    } else {
        draw_text_blend(buffer, width, height, text, x, y, color, size, font);
    }
}

fn draw_text_blend(
    buffer: *mut u32,
    width: &usize,
    height: &usize,
    text: &str,
    x: usize,
    y: usize,
    color: u32,
    size: f32,
    font: &Font,
) {
    let mut pen_x = x;
    let pen_y = y;

    let rounded_size_key = round_float_key(size);
    let font_metrics = font.horizontal_line_metrics(size).unwrap();

    for ch in text.chars() {
        // Try to get the glyph from cache first
        let (metrics, bitmap) = {
            let cache = get_glyph_cache().borrow();
            cache.get(&(ch, rounded_size_key)).cloned()
        }
        .unwrap_or_else(|| {
            let rasterized = font.rasterize(ch, size);

            // Insert into cache
            let mut cache_mut = get_glyph_cache().borrow_mut();
            cache_mut.insert((ch, rounded_size_key), rasterized.clone());

            rasterized
        });

        // Draw each character into the buffer
        for gy in 0..metrics.height {
            for gx in 0..metrics.width {
                let px = pen_x + gx;
                // Correcting for letter height
                let py = pen_y
                    + gy
                    + (font_metrics.ascent - metrics.height as f32) as usize;

                if px < *width && py < *height {
                    let index = py * width + px;
                    let alpha = bitmap[gy * metrics.width + gx]; // Alpha (0-255)

                    if alpha > 0 {
                        unsafe {
                            let bg = *buffer.add(index);
                            // Extract RGBA
                            let (br, bg, bb, ba) = (
                                (bg >> 24) & 0xFF,
                                (bg >> 16) & 0xFF,
                                (bg >> 8) & 0xFF,
                                bg & 0xFF,
                            );
                            let (tr, tg, tb, ta) = (
                                (color >> 24) & 0xFF,
                                (color >> 16) & 0xFF,
                                (color >> 8) & 0xFF,
                                color & 0xFF,
                            );

                            // Alpha blending
                            let inv_alpha: u8 = 255 - alpha;
                            let nr = ((tr as u16 * alpha as u16
                                + br as u16 * inv_alpha as u16)
                                / 255)
                                as u8;
                            let ng = ((tg as u16 * alpha as u16
                                + bg as u16 * inv_alpha as u16)
                                / 255)
                                as u8;
                            let nb = ((tb as u16 * alpha as u16
                                + bb as u16 * inv_alpha as u16)
                                / 255)
                                as u8;
                            let na = ((ta as u16 * alpha as u16
                                + ba as u16 * inv_alpha as u16)
                                / 255)
                                as u8;

                            draw_pixel(
                                buffer,
                                width,
                                height,
                                px,
                                py,
                                (nr as u32) << 24
                                    | (ng as u32) << 16
                                    | (nb as u32) << 8
                                    | na as u32,
                            );
                        }
                    }
                }
            }
        }

        // Advance the cursor position
        pen_x += metrics.advance_width as usize;
    }
}

fn draw_text_no_blend(
    buffer: *mut u32,
    width: &usize,
    height: &usize,
    text: &str,
    x: usize,
    y: usize,
    color: u32,
    size: f32,
    font: &Font,
) {
    let mut pen_x = x;
    let pen_y = y;
    let font_metrics = font.horizontal_line_metrics(size).unwrap();
    let ascent = font_metrics.ascent as usize;

    let rounded_size_key = round_float_key(size);

    for ch in text.chars() {
        // Try to get the glyph from cache first
        let cached_glyph = {
            let cache = get_glyph_cache().borrow();
            cache.get(&(ch, rounded_size_key)).cloned()
        };

        // If not in cache, rasterize and insert
        let (metrics, bitmap) = cached_glyph.unwrap_or_else(|| {
            let rasterized = font.rasterize(ch, size);

            // Insert into cache
            let mut cache_mut = get_glyph_cache().borrow_mut();
            cache_mut.insert((ch, rounded_size_key), rasterized.clone());

            rasterized
        });

        let offset_y = ascent.saturating_sub(metrics.height);
        let w = metrics.width;
        let h = metrics.height;
        let advance_x = metrics.advance_width as usize;

        for gy in 0..h {
            let py = pen_y + gy + offset_y;
            if py >= *height {
                continue;
            }

            let row_start = gy * w;
            for gx in 0..w {
                let px = pen_x + gx;
                if px >= *width {
                    continue;
                }

                if bitmap[row_start + gx] > 0 {
                    draw_pixel(buffer, width, height, px, py, color);
                }
            }
        }
        pen_x += advance_x;
    }
}

#[inline(always)]
fn draw_pixel(
    buffer: *mut u32,
    width: &usize,
    height: &usize,
    x: usize,
    y: usize,
    color: u32,
) {
    if FAST_RENDER {
        draw_pixel_unsafe(buffer, width, x, y, color);
    } else {
        draw_pixel_safe(buffer, width, height, x, y, color);
    }
}

#[inline(always)]
fn draw_pixel_safe(
    buffer: *mut u32,
    width: &usize,
    height: &usize,
    x: usize,
    y: usize,
    color: u32,
) {
    // if x < 0 || y < 0 {
    //     return;
    // }
    if x >= *width || y >= *height {
        return;
    }
    draw_pixel_unsafe(buffer, width, x, y, color);
}

#[inline(always)]
fn draw_pixel_unsafe(
    buffer: *mut u32,
    width: &usize,
    x: usize,
    y: usize,
    color: u32,
) {
    unsafe {
        *buffer.add(y * width + x) = color;
    }
}

// if false {
//     let found = get_pixel(buffer, width, height, x, y);
//     if found != 0 {
//         return;
//     }
// }
#[inline(always)]
fn get_pixel(
    buffer: &Vec<u32>,
    width: &isize,
    height: &isize,
    x: isize,
    y: isize,
) -> u32 {
    if x < 0 || y < 0 {
        return 0;
    }
    if x >= *width || y >= *height {
        return 0;
    }
    let index = y * width + x;
    return buffer[index as usize];
}
fn visible_tiles(
    outer: (i32, i32, i32, i32),
    holes: &[(i32, i32, i32, i32)],
) -> HashSet<(i32, i32)> {
    let (x0, y0, x1, y1) = outer;
    let mut visible = HashSet::new();

    // Insert all tiles from the outer square
    for y in y0..y1 {
        for x in x0..x1 {
            visible.insert((x, y));
        }
    }

    // Remove tiles from each hole
    for &(hx0, hy0, hx1, hy1) in holes {
        for y in hy0..hy1 {
            for x in hx0..hx1 {
                visible.remove(&(x, y));
            }
        }
    }

    visible
}

fn render_block(
    block: &Block,
    camera: &Camera,
    buffer: *mut u32,
    block_colors: &Vec<u32>,
    width: &isize,
    height: &isize,
    font: &Font,
) {
    render_block_internal(
        block,
        block.x.get() as isize,
        block.y.get() as isize,
        camera,
        buffer,
        block_colors[block.block_color_id],
        &(*width as usize),
        &(*height as usize),
        font,
    );
}
#[inline(always)]
fn get_top_most_block_id_or_self(blocks: &Vec<Block>, index: usize) -> ID {
    // Is this broken???????
    let block = &blocks[index];
    if block.connected_top.get().is_some() {
        return block.connected_top.get().unwrap();
    }
    return block.id;
}

fn handle_connection_and_render_ghost_block(
    buffer: *mut u32,
    width: &usize,
    height: &usize,
    camera: &Camera,
    blocks: &Vec<Block>,
    font: &Font,
    selected: &Option<usize>,
    snap_distance: f32,
) {
    if selected.is_some() {
        // Connect to block above
        let possible = get_closest_connection(
            &blocks,
            blocks[selected.unwrap()].x.get() as f32,
            blocks[selected.unwrap()].y.get() as f32,
            snap_distance,
            *selected,
            true,
        );
        let block = &blocks[selected.unwrap()];
        block.possible_connection_above.set(None);

        if possible.is_some() {
            let above_block = &blocks[possible.unwrap()];
            // TODO: Fix this -> This always returns false
            if is_there_a_loop_in_block_connections_for_block_internal(
                &blocks,
                get_top_most_block_id_or_self(
                    blocks,
                    index_by_block_id(above_block.id, blocks).unwrap(),
                ),
                &mut Vec::from([block.id]),
            ) {
                //println!("Loop avoided");
                return;
            } else {
                // Save block, only if it is not a loop
                block.possible_connection_above.set(Some(above_block.id));
            }

            if !is_block_visible_on_screen(
                &above_block,
                camera,
                &(*width as isize),
                &(*height as isize),
            ) {
                return;
            }
            render_block_internal(
                &above_block,
                above_block.x.get() as isize,
                above_block.y.get() as isize
                    + above_block.height.get() as isize,
                camera,
                buffer,
                mirl::graphics::rgb_to_u32(100, 100, 100),
                width,
                height,
                font,
            );
        }
    }
}
fn is_in_any_hole(
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
fn render_block_internal(
    block: &Block,
    origin_x: isize,
    origin_y: isize,
    camera: &Camera,
    buffer: *mut u32,
    block_color: u32,
    width: &usize,
    height: &usize,
    font: &Font,
) {
    // let block_x = block.x.get() as isize;
    // let block_y = block.y.get() as isize;
    // let block_w = block.width.get() as isize;
    // let block_h = block.height.get() as isize;

    // let to_draw = visible_tiles(
    //     (
    //         block_x as i32,
    //         block_y as i32,
    //         (block_x + block_w) as i32,
    //         (block_y + block_h) as i32,
    //     ),
    //     &[],
    // );

    // for (x, y) in to_draw {
    //     let screen_x = (x as isize - camera.x) as usize;
    //     let screen_y = (y as isize - camera.y) as usize;

    //     draw_pixel(buffer, width, height, screen_x, screen_y, block_color);
    // }

    let x0 = block.x.get() as isize;
    let y0 = block.y.get() as isize;
    let x1 = block.x.get() as isize + block.width.get() as isize;
    let y1 = block.y.get() as isize + block.height.get() as isize;
    let holes = &[(1, 1, 40, 20)];

    for y in y0..y1 {
        for x in x0..x1 {
            // HOLES DO NOT MOVE WITH BLOCK -> FIX THAT FUTURE ME
            if is_in_any_hole(x - camera.x, y - camera.y, holes) {
                continue;
            }

            draw_pixel(
                buffer,
                width,
                height,
                (x - camera.x) as usize,
                (y - camera.y) as usize,
                block_color,
            );
        }
    }

    draw_text(
        buffer,
        &(*width as usize),
        &(*height as usize),
        &block.name,
        (origin_x - camera.x) as usize,
        (origin_y - camera.y) as usize,
        mirl::graphics::rgb_to_u32(255, 0, 0),
        20.0,
        font,
    );
}

#[inline(always)]
fn is_block_visible_on_screen(
    block: &Block,
    camera: &Camera,
    width: &isize,
    height: &isize,
) -> bool {
    return is_reqtuctangle_visible_on_screen(
        block.x.get() as f32,
        block.y.get() as f32,
        block.width.get() as f32,
        block.height.get() as f32,
        camera,
        width,
        height,
    );
}

#[inline(always)]
fn is_reqtuctangle_visible_on_screen(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    camera: &Camera,
    buffer_width: &isize,
    buffer_height: &isize,
) -> bool {
    if FAST_LOGIC {
        return is_reqtuctangle_visible_on_screen_cheap(
            x,
            y,
            width,
            height,
            camera,
            buffer_width,
            buffer_height,
        );
    } else {
        return is_reqtuctangle_visible_on_screen_expensive(
            x,
            y,
            width,
            height,
            camera,
            buffer_width,
            buffer_height,
        );
    }
}

#[inline(always)]
fn is_reqtuctangle_visible_on_screen_expensive(
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
    if is_point_in_requctangle(x, y, cam_x, cam_y, cam_width, cam_height) {
        return true;
    }
    if is_point_in_requctangle(x2, y, cam_x, cam_y, cam_width, cam_height) {
        return true;
    }
    if is_point_in_requctangle(x, y2, cam_x, cam_y, cam_width, cam_height) {
        return true;
    }
    if is_point_in_requctangle(x2, y2, cam_x, cam_y, cam_width, cam_height) {
        return true;
    }
    return false;
}

#[inline(always)]
fn is_reqtuctangle_visible_on_screen_cheap(
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
    if !is_point_in_requctangle(x, y, cam_x, cam_y, cam_width, cam_height) {
        return false;
    }
    if !is_point_in_requctangle(x2, y, cam_x, cam_y, cam_width, cam_height) {
        return false;
    }
    if !is_point_in_requctangle(x, y2, cam_x, cam_y, cam_width, cam_height) {
        return false;
    }
    if !is_point_in_requctangle(x2, y2, cam_x, cam_y, cam_width, cam_height) {
        return false;
    }
    return true;
}

// Misc
fn handle_and_render_on_screen(
    buffer: *mut u32,
    width: &usize,
    height: &usize,
    camera: &Camera,
    blocks: &mut Vec<Block>,
    block_colors: &Vec<u32>,
    font: &Font,
) {
    let now_width = *width as isize;
    let now_height = *height as isize;

    let block_ids: Vec<usize> = (0..blocks.len()).collect();

    // Reverse block order in order for overdraw to to its job in our favor
    for id in block_ids {
        if blocks[id].recently_moved.get() {
            move_block_to_connected(blocks, &Some(id));
        }
        let block = &blocks[id];

        if !is_block_visible_on_screen(block, camera, &now_width, &now_height) {
            continue;
        }
        render_block(
            block,
            camera,
            buffer,
            block_colors,
            &now_width,
            &now_height,
            font,
        );
    }
}
#[inline(always)]
fn subtract_tuple(one: (f32, f32), two: (f32, f32)) -> (f32, f32) {
    (one.0 - two.0, one.1 - two.1)
}
#[inline(always)]
fn reorder_element<T>(vec: &mut Vec<T>, from: usize, to: usize) {
    if from != to && from < vec.len() && to < vec.len() {
        let item = vec.remove(from);
        vec.insert(to, item)
    }
}
#[inline(always)]
fn is_point_in_requctangle(
    x: f32,
    y: f32,
    origin_x: f32,
    origin_y: f32,
    width: f32,
    height: f32,
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
    return true;

    // let temp_x = x - origin_x;
    // let temp_y = y - origin_y;
    // if !(x > 0.0 && x < width) {
    //     return false;
    // }
    // if !(y > 0.0 && y < height) {
    //     return false;
    // }

    // return true;
}

fn get_length_of_text_in_font(text: &str, font: &Font) -> f32 {
    let mut length = 0.0;
    for ch in text.chars() {
        let (metrics, _) = font.rasterize(ch, 20.0);
        length += metrics.advance_width;
    }
    return length;
}
// Block stuff
fn get_block_id_above(
    blocks: &Vec<Block>,
    pos_x: f32,
    pos_y: f32,
) -> Option<usize> {
    for block_id in 0..blocks.len() {
        let block = &blocks[block_id];
        if is_point_in_requctangle(
            pos_x,
            pos_y,
            block.x.get() as f32,
            block.y.get() as f32 + block.height.get(),
            block.width.get() as f32,
            block.height.get() as f32,
        ) {
            return Some(block_id);
        }
    }
    return None;
}
fn get_closest_connection(
    blocks: &Vec<Block>,
    pos_x: f32,
    pos_y: f32,
    max_distance: f32,
    blacklisted: Option<usize>,
    top: bool,
) -> Option<usize> {
    if FAST_LOGIC {
        return get_any_block_in_distance(
            blocks,
            pos_x,
            pos_y,
            max_distance,
            blacklisted,
            top,
        );
    } else {
        get_closest_block_in_distance(
            blocks,
            pos_x,
            pos_y,
            max_distance,
            blacklisted,
            top,
        )
    }
}

fn get_closest_block_in_distance(
    blocks: &[Block],
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
        let distance = get_distance_between_positions(
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

    return closest;
}
fn get_any_block_in_distance(
    blocks: &Vec<Block>,
    pos_x: f32,
    pos_y: f32,
    max_distance: f32,
    blacklisted: Option<usize>,
    top: bool,
) -> Option<usize> {
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
            if get_distance_between_positions(
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
fn get_block_id_under_point(
    blocks: &Vec<Block>,
    pos_x: f32,
    pos_y: f32,
) -> Option<usize> {
    for block_id in 0..blocks.len() {
        let block = &blocks[block_id];
        if is_point_in_requctangle(
            pos_x,
            pos_y,
            block.x.get() as f32,
            block.y.get() as f32,
            block.width.get() as f32,
            block.height.get() as f32,
        ) {
            return Some(block_id);
        }
    }
    return None;
}

// Global id counter
#[inline(always)]
fn get_global_block_id() -> usize {
    return GLOBAL_BLOCK_COUNTER.lock().unwrap().clone();
}
#[inline(always)]
fn increment_global_block_id() -> usize {
    let mut counter = GLOBAL_BLOCK_COUNTER.lock().unwrap();
    if *counter == usize::MAX {
        panic!(
            "Ran out of block ids -> tried to create more than {} blocks (Maximal value of usize, honestly impressive. Wait, isn't that like a petabyte of memory? How has your computer not crashed before this function call?)",
            usize::MAX
        );
    }
    *counter += 1;
    return *counter;
}

#[inline(always)]
fn get_distance_between_positions(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    if FAST_LOGIC {
        return get_approx_distance(x1, y1, x2, y2);
    } else {
        return get_distance_between_positions_accurate(x1, y1, x2, y2);
    }
}
#[inline(always)]
fn get_distance_between_positions_accurate(
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
) -> f32 {
    return ((x1 - x2) * (x1 - x2) + (y1 - y2) * (y1 - y2)).sqrt();
}
#[inline(always)]
fn get_approx_distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let dx = (x1 - x2).abs();
    let dy = (y1 - y2).abs();
    dx.max(dy) + 0.41 * dx.min(dy)
}
fn add_item_to_max_sized_list(list: &mut Vec<u64>, max_size: usize, item: u64) {
    list.push(item);
    if list.len() < max_size {
        return;
    }
    let to_remove = list.len() - max_size;
    for _ in 0..to_remove {
        list.remove(0);
    }
}

#[inline(always)]
fn index_by_block_id(id: ID, blocks: &Vec<Block>) -> Option<usize> {
    for block_id in 0..blocks.len() {
        if blocks[block_id].id == id {
            return Some(block_id);
        }
    }
    return None;
}

fn is_there_a_loop_in_block_connections_for_block(
    blocks: &Vec<Block>,
    block_index: ID,
) -> bool {
    let mut already_checked = Vec::new();
    return is_there_a_loop_in_block_connections_for_block_internal(
        blocks,
        block_index,
        &mut already_checked,
    );
}
fn is_there_a_loop_in_block_connections_for_block_internal(
    blocks: &Vec<Block>,
    block_id: ID,
    already_checked: &mut Vec<ID>,
) -> bool {
    // I don't think this function works quite right
    if already_checked.contains(&block_id) {
        return true;
    }
    already_checked.push(block_id);
    let block = &blocks[index_by_block_id(block_id, blocks).unwrap()];
    if block.connected_below.get().is_some() {
        return is_there_a_loop_in_block_connections_for_block_internal(
            blocks,
            block.connected_below.get().unwrap(),
            already_checked,
        );
    }
    // if block.connected_below.get().is_some() {
    //     return is_there_a_loop_in_block_connections_for_block(
    //         blocks,
    //         block.connected_below.get().unwrap(),
    //         already_checked,
    //     );
    // }
    return false;
}
#[inline(always)]
fn custom_join(vector: Vec<ID>, separator: &str) -> String {
    let mut out = "".to_string();
    let length = vector.len();
    for idx in 0..length {
        let item = vector[idx].to_string();
        out = out + &item;
        if idx != length - 1 {
            out = out + separator;
        }
    }
    return out;
}

fn get_ids_connected_to_block(
    top_most_block_id: ID,
    blocks: &Vec<Block>,
    found: &mut Vec<ID>,
) -> Vec<ID> {
    // FIX THIS FUNCTION FOR GOD SAKE WHY ARE THERE MUTLIPLE IDS??????????????????
    let block = &blocks[index_by_block_id(top_most_block_id, &blocks).unwrap()];

    if found.contains(&block.id) {
        println!("{} -> {}", block.id, block.connected_below.get().unwrap());
        panic!(
            "Infinite loop found with these ids {} with current {} (len({}))",
            custom_join(found.to_vec(), ", "),
            top_most_block_id,
            found.len()
        )
    }
    found.push(top_most_block_id);
    if block.connected_below.get().is_some() {
        let sub = get_ids_connected_to_block(
            block.connected_below.get().unwrap(),
            blocks,
            found,
        );

        found.extend(sub)
    }

    // Filter out multiple of an id -> WE SHOULD NOT NEED TO DO THIS
    let mut seen = std::collections::HashSet::new();

    found.retain(|x| seen.insert(*x));

    return found.to_vec();
}
#[inline(always)]
fn index_by_block_ids(
    blocks: &Vec<Block>,
    ids: &Vec<ID>,
) -> Vec<Option<usize>> {
    let mut return_list = Vec::new();
    for id in ids {
        return_list.push(index_by_block_id(*id, blocks))
    }
    return return_list;
}
fn get_total_height_of_blocks(
    blocks: &Vec<Block>,
    indexes: Vec<Option<usize>>,
) -> f32 {
    // For optimization, remove the Option<>
    let mut height: f32 = 0.0;
    for idx in indexes {
        if idx.is_none() {
            continue;
        }
        let block = &blocks[idx.unwrap()];
        height += block.height.get();
    }
    return height;
}
fn draw_circle(
    buffer: *mut u32,
    width: &usize,
    height: &usize,
    pos_x: usize,
    pos_y: usize,
    radius: isize,
    color: u32,
) {
    let mut x = 0;
    let mut y = 0 - radius;
    let mut p = -radius;

    while (x) < (-y) {
        if p > 0 {
            y += 1;
            p += 2 * (x + y) + 1
        } else {
            p += 2 * x + 1
        }
        let temp_x = x as usize;
        let temp_y = y as usize;
        draw_pixel(
            buffer,
            width,
            height,
            pos_x + temp_x,
            pos_y + temp_y,
            color,
        );
        draw_pixel(
            buffer,
            width,
            height,
            pos_x - temp_x,
            pos_y + temp_y,
            color,
        );
        draw_pixel(
            buffer,
            width,
            height,
            pos_x + temp_x,
            pos_y - temp_y,
            color,
        );
        draw_pixel(
            buffer,
            width,
            height,
            pos_x - temp_x,
            pos_y - temp_y,
            color,
        );
        draw_pixel(
            buffer,
            width,
            height,
            pos_x + temp_y,
            pos_y + temp_x,
            color,
        );
        draw_pixel(
            buffer,
            width,
            height,
            pos_x + temp_y,
            pos_y - temp_x,
            color,
        );
        draw_pixel(
            buffer,
            width,
            height,
            pos_x - temp_y,
            pos_y + temp_x,
            color,
        );
        draw_pixel(
            buffer,
            width,
            height,
            pos_x - temp_y,
            pos_y - temp_x,
            color,
        );

        x += 1
    }
}
fn get_usize_out_of_option_list(list: Vec<Option<usize>>) -> Vec<usize> {
    let mut new = Vec::new();
    for l in list {
        new.push(l.unwrap())
    }
    return new;
}

fn move_block_to_connected(blocks: &mut Vec<Block>, index: &Option<usize>) {
    if index.is_none() {
        return;
    }
    let block = &blocks[index.unwrap()];
    let top_block_id = block.connected_top.get();
    if top_block_id.is_none() {
        return;
    }
    let top_block =
        &blocks[index_by_block_id(top_block_id.unwrap(), blocks).unwrap()];
    let block_query = get_ids_connected_to_block(
        top_block_id.unwrap(),
        blocks,
        &mut Vec::new(),
    );
    // THIS ERRORS vvv
    let own_block_id_index = block_query
        .iter()
        .position(|&id| id == block.id)
        .expect("FIX ME GOD DAMN IT");

    let blocks_to_offset_with =
        &block_query[0..own_block_id_index + 1].to_vec();

    //println!(">{:?}", block_query);

    let total_offset = get_total_height_of_blocks(
        blocks,
        index_by_block_ids(blocks, blocks_to_offset_with),
    );

    block.x.set(top_block.x.get());
    block.y.set(
        total_offset as u16 + top_block.y.get() - block.height.get() as u16,
    );
    block.recently_moved.set(false);
    // blocks[above_block.unwrap()].x.set(blocks[selected.unwrap()].x.get());
    // blocks[above_block.unwrap()].y.set(
    //     blocks[selected.unwrap()].y.get()
    //         + blocks[selected.unwrap()].height.get() as u16,
    // );
}
#[inline(always)]
fn get_difference_of_values_in_percent(value1: f64, value2: f64) -> f64 {
    if value1 == 0.0 {
        return 100.0; // Avoid division by zero, assuming 100% difference
    }
    ((value2 - value1) / value1).abs() * 100.0
}

fn handle_selected_and_mouse<R: platform::shared::Framework>(
    mouse_outside: bool,
    mouse_down: bool,
    last_mouse_down: bool,
    blocks: &mut Vec<Block>,
    camera: &mut Camera,
    mouse_pos: (f32, f32),
    mouse_delta: (f32, f32),
    framework: &R,
    scroll_multiplier: f32,
    selected: Option<usize>,
) -> Option<usize> {
    // There are too many problems with dealing with null when the mouse is outside the window, so instead we just check if the mouse is with in the window :)
    let mut selected = selected;
    if !mouse_outside {
        if mouse_down {
            if !last_mouse_down {
                selected = get_block_id_under_point(
                    &blocks,
                    mouse_pos.0 + camera.x as f32,
                    mouse_pos.1 + camera.y as f32,
                );
            }
            if selected.is_some() {
                let list = get_ids_connected_to_block(
                    get_top_most_block_id_or_self(&blocks, selected.unwrap()),
                    &blocks,
                    &mut Vec::new(),
                );
                // When this block is selected, disconnect it from the blocks above
                blocks[selected.unwrap()].connected_top.set(None);

                // find position of element with value blocks[selected.unwrap()].id
                let split_point = list
                    .iter()
                    .position(|block| *block == blocks[selected.unwrap()].id);

                // "Split point" -> below gets moved, above stays put, split point is selected block
                if split_point.is_none() {
                    panic!("The block thought it was hanging from a block structure that isn't actually connected to this block anymore (Currently happening when replacing a block)")
                }

                if split_point.unwrap() > 0 {
                    // Remove the connected tag from the block this block is connected to (But only if that block above exists!)
                    let block_above = &blocks[index_by_block_id(
                        list[split_point.unwrap() - 1],
                        &blocks,
                    )
                    .unwrap()];
                    block_above.connected_below.set(None);
                }

                // Tell blocks below that they have been moved
                for id in list[split_point.unwrap()..].iter() {
                    blocks[index_by_block_id(*id, &blocks).unwrap()]
                        .recently_moved
                        .set(true);
                }
            }
        } else {
            // Connect block previously selected if possible
            if selected.is_some() {
                let block = &blocks[selected.unwrap()];
                if block.possible_connection_above.get().is_some() {
                    let connection_id_above =
                        block.possible_connection_above.get().unwrap();

                    if is_there_a_loop_in_block_connections_for_block_internal(
                        &blocks,
                        get_top_most_block_id_or_self(
                            &blocks,
                            index_by_block_id(
                                block.possible_connection_above.get().unwrap(),
                                &blocks,
                            )
                            .unwrap(),
                        ),
                        &mut Vec::from([blocks[selected.unwrap()].id]),
                    ) {
                        panic!("Loop detected");
                    }
                    // Get above block
                    let block_above_index =
                        index_by_block_id(connection_id_above, &blocks)
                            .unwrap();
                    let above_block = &blocks[block_above_index];
                    // Tell current block to connect above
                    block.connected_top.set(Some(
                        get_top_most_block_id_or_self(
                            &blocks,
                            block_above_index,
                        ),
                    ));
                    // WE CURRENTLY JUST REPLACE THE CURRENT BLOCK-> THESE BLOCKS SHOULD BE APPENDED TO THE CURRENT BLOCKS AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAHHHHHHHHHHH
                    above_block.connected_below.set(Some(block.id));
                }

                // Set current possible above block to none
                block.possible_connection_above.set(None);
                block.recently_moved.set(true);
            }
            selected = None;
        }

        if mouse_down {
            if let Some(mut idx) = selected {
                reorder_element(blocks, idx, 0);
                idx = 0;
                selected = Some(0);
                blocks[idx]
                    .x
                    .set((blocks[idx].x.get() as f32 + mouse_delta.0) as u16);
                blocks[idx]
                    .y
                    .set((blocks[idx].y.get() as f32 + mouse_delta.1) as u16);
            } else {
                camera.x -= mouse_delta.0 as isize;
                camera.y -= mouse_delta.1 as isize;
            }
        }

        // Mouse wheel movement
        let mouse_wheel_temp = framework.get_mouse_scroll();
        if mouse_wheel_temp.is_some() {
            let extra_mul;
            if framework.is_key_down(platform::KeyCode::RightShift) {
                extra_mul = 10.0;
            } else {
                extra_mul = 1.0;
            }
            if framework.is_key_down(platform::KeyCode::LeftControl) {
                camera.z -= mouse_wheel_temp.unwrap().1 * extra_mul;
            } else {
                if framework.is_key_down(platform::KeyCode::LeftShift) {
                    camera.y -= (mouse_wheel_temp.unwrap().0
                        * scroll_multiplier
                        * extra_mul) as isize;
                    camera.x -= (mouse_wheel_temp.unwrap().1
                        * scroll_multiplier
                        * extra_mul) as isize;
                } else {
                    camera.x -= (mouse_wheel_temp.unwrap().0
                        * scroll_multiplier
                        * extra_mul) as isize;
                    camera.y -= (mouse_wheel_temp.unwrap().1
                        * scroll_multiplier
                        * extra_mul) as isize;
                }
            }
        }
    }
    return selected;
}
use serde_json;
pub fn parse_translations(
    csv_data: &str,
    lang: &str,
) -> Result<Option<(Vec<String>, Vec<String>)>, Box<dyn Error>> {
    let mut rdr =
        ReaderBuilder::new().has_headers(true).from_reader(csv_data.as_bytes());

    let headers = rdr.headers()?.clone();
    let lang_index = headers.iter().position(|h| h == lang);
    let key_index = headers.iter().position(|h| h == "translation_key");

    if let (Some(k_idx), Some(l_idx)) = (key_index, lang_index) {
        let mut keys = Vec::new();
        let mut values = Vec::new();

        for result in rdr.records() {
            let record = result?;
            keys.push(record.get(k_idx).unwrap_or("").to_string());
            values.push(record.get(l_idx).unwrap_or("").to_string());
        }

        Ok(Some((keys, values)))
    } else {
        Ok(None)
    }
}
use csv::ReaderBuilder;
use std::error::Error;

fn load_blocks<F: FileSystem>(
    file_system: &F,
    block_output_types: &mut Vec<String>,
    block_output_colors: &mut Vec<u32>,
    translation: &mut HashMap<String, String>,
    font: &Font,
) -> Vec<Block> {
    let path =
        r"C:\personal\games\minecraft\Automated\generation_lib\procedures";
    let mut blocks = Vec::new();

    let all_plugin_folders = file_system.get_folders_in_folder(path);
    println!("{:?}", all_plugin_folders);
    for plugin_folder in all_plugin_folders {
        let plugin_path = file_system.join(path, &plugin_folder);
        let setting_file = file_system.join(&plugin_path, "settings.json");
        let translation_file =
            file_system.join(&plugin_path, "translation.csv");

        let translation_data = file_system.get_file_contents(&translation_file);
        let extracted =
            parse_translations(&translation_data, "en").unwrap().unwrap();

        for i in 0..extracted.0.len() {
            translation.insert(extracted.0[i].clone(), extracted.1[i].clone());
        }
        // translation_keys.extend(extracted.0);
        // translation_values.extend(extracted.1);

        let settings_json: serde_json::Value =
            serde_json::from_str(&file_system.get_file_contents(&setting_file))
                .expect("Unable to load json settings file");
        let pre_blocks = settings_json
            .get("blocks")
            .ok_or("Error")
            .expect("Error unwrapping");
        for block in
            pre_blocks.as_array().ok_or("Error").expect("Error unwrapping")
        {
            println!(
                "{}",
                block.get("name").ok_or("Error").expect("Error unwrapping")
            );
            let block_type =
                block.get("type").ok_or("Error").expect("Error unwrapping");
            if block_type != "action" {
                println!(
                    "Skipping {} because it's not an action block",
                    block_type
                );
                continue;
            }
            let internal_name: String = block
                .get("name")
                .expect("Missing name")
                .as_str()
                .expect("Error unwrapping")
                .to_string();

            let output = block
                .get("output")
                .expect(&format!("Missing output for {}", internal_name))
                .as_str()
                .expect("Error unwrapping")
                .to_string();

            if !block_output_types.contains(&output) {
                block_output_types.push(output.clone());
                block_output_colors.push(generate_random_color());
            }

            let name = translation.get(&internal_name).unwrap();

            let block = Block::new(
                name.clone(),
                internal_name,
                0,
                0,
                0,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                output,
                Vec::new(),
                block_output_types,
                font,
            );
            blocks.push(block);
        }
    }
    return blocks;
}
#[inline(always)]
fn generate_random_color() -> u32 {
    return getrandom::u32().expect("Unable to generate random color");
}
// #[inline(always)]
// fn title_case(s: &str) -> String {
//     s.split_whitespace()
//         .map(|word| {
//             let mut chars = word.chars();
//             match chars.next() {
//                 Some(first) => {
//                     first.to_uppercase().collect::<String>() + chars.as_str()
//                 }
//                 None => String::new(),
//             }
//         })
//         .collect::<Vec<_>>()
//         .join(" ")
// }
// fn internal_name_to_name(internal_name: &str) -> String {
//     let mut name = internal_name.to_string();
//     name = name.replace("_", " ");
//     //title case
//     name = title_case(&name);
//     return name;
// }
// Goal: Render 1000 Blocks at >=60 fps
// ------------------------------------------< SETTINGS >------------------------------------------

const FAST_RENDER: bool = true; // ~10% <-> ~40% Faster rendering
const FAST_LOGIC: bool = false || FAST_RENDER; // Render methods depend on cheap logic

// ------------------------------------------------------------------------------------------------
// Current:
// Optimized (10_000):
// FAST_RENDER OFF: 18 fps
// FAST_RENDER ON: 25 fps

static GLOBAL_BLOCK_COUNTER: once_cell::sync::Lazy<std::sync::Mutex<usize>> =
    once_cell::sync::Lazy::new(|| std::sync::Mutex::new(0));

use crate::platform;

pub fn main_loop<
    R: platform::shared::Framework,
    F: platform::shared::FileSystem,
>(
    mut framework: R,
    file_system: F,
) {
    //platform::log("Entered main loop");
    let snap_distance = 70.0;
    let scroll_multiplier = 5.0;
    let max_fps = 60;
    let target_frame_delta = mirl::time::MILLIS_PER_SEC / max_fps; // Time for one frame at the target FPS
    let mut frame_start;

    let mut delta_time;
    let mut buffer: Vec<u32>;
    let mut fps;

    let mut mouse_pos = framework.get_mouse_position().unwrap_or((0.0, 0.0));
    let mut mouse_delta;

    let mut camera = Camera {
        x: (u16::MAX / 2) as isize,
        y: (u16::MAX / 2) as isize,
        z: 1.0,
    };

    let mut blocks: Vec<Block> = Vec::new();

    //let mut color_names: Vec<String> = Vec::new();
    let mut block_output_color_rgb: Vec<u32> = Vec::new();
    let mut block_output_color_names: Vec<String> = Vec::new();

    //color_names.push("bool".to_string());
    block_output_color_rgb.push(mirl::graphics::rgb_to_u32(50, 80, 255));
    block_output_color_names.push("bool".to_string());
    let font = platform::load_font("src/inter.ttf");

    let mut translation = HashMap::new();

    blocks.extend(load_blocks(
        &file_system,
        &mut block_output_color_names,
        &mut block_output_color_rgb,
        &mut translation,
        &font,
    ));
    // for id in 0..1000 {
    //     blocks.push(Block::new(
    //         format!("new block {}", id + 1),
    //         getrandom::u32().expect("Unable to generate random color") as i16
    //             / 1000,
    //         getrandom::u32().expect("Unable to generate random color") as i16
    //             / 1000,
    //         0,
    //         Vec::new(),
    //         Vec::new(),
    //         Vec::new(),
    //         Vec::new(),
    //         "bool".to_string(),
    //         Vec::new(),
    //         &block_output_color_names,
    //         &font,
    //     ))
    // }

    let mut last_mouse_down;
    let mut mouse_down_temp;
    let mut mouse_down = false;

    let mut fps_list: Vec<u64> = Vec::new();

    let mut selected: Option<usize> = None;

    let mut mouse_outside;
    let mut stable_fps: u64;

    let (width, height) = framework.get_size();

    buffer = mirl::render::get_empty_buffer(width, height);
    let buffer_pointer: *mut u32 = buffer.as_mut_ptr();
    let total_window_size = width * height;

    //platform::log("Starting main loop");
    frame_start = framework.get_time();
    while framework.is_open() {
        mirl::render::clear_screen(buffer_pointer, total_window_size);

        // if (buffer_pointer as usize) % 16 == 0 {
        //     println!("Buffer is 16-byte aligned");
        // } else {
        //     println!("Buffer is NOT 16-byte aligned");
        // }
        // Mouse stuff and block(/camera) selection/movement
        mouse_delta = mouse_pos;
        mouse_pos = framework.get_mouse_position().unwrap_or(mouse_pos);

        mouse_delta = subtract_tuple(mouse_pos, mouse_delta);
        mouse_outside = !is_point_in_requctangle(
            mouse_pos.0,
            mouse_pos.1,
            0.0,
            0.0,
            width as f32,
            height as f32,
        );

        mouse_down_temp = mouse_down;
        mouse_down = framework.is_mouse_down(platform::MouseButton::Left);
        last_mouse_down = mouse_down_temp && mouse_down;

        selected = handle_selected_and_mouse(
            mouse_outside,
            mouse_down,
            last_mouse_down,
            &mut blocks,
            &mut camera,
            mouse_pos,
            mouse_delta,
            &framework,
            scroll_multiplier,
            selected,
        );

        //############################################

        handle_and_render_on_screen(
            buffer_pointer,
            &width,
            &height,
            &camera,
            &mut blocks,
            &block_output_color_rgb,
            &font,
        );
        handle_connection_and_render_ghost_block(
            buffer_pointer,
            &width,
            &height,
            &camera,
            &blocks,
            &font,
            &selected,
            snap_distance,
        );
        let mouse_size = 4;
        let mouse_size_half = mouse_size as f32 / 2.0;

        if is_reqtuctangle_visible_on_screen(
            mouse_pos.0 - mouse_size_half,
            mouse_pos.1 - mouse_size_half,
            mouse_size as f32,
            mouse_size as f32,
            &camera,
            &(width as isize),
            &(width as isize),
        ) {
            draw_circle(
                buffer_pointer,
                &width,
                &height,
                mouse_pos.0 as usize,
                mouse_pos.1 as usize,
                mouse_size,
                rgb_to_u32(100, 20, 200),
            );
        }
        //############################################
        framework.update(&buffer); // "Unable to update window :("
        delta_time = frame_start.get_elapsed_time();
        frame_start = framework.get_time();

        if delta_time != 0 {
            fps = mirl::time::MILLIS_PER_SEC as u64 / delta_time;
        } else {
            fps = u64::MAX;
        }

        add_item_to_max_sized_list(&mut fps_list, fps as usize / 4, fps as u64);
        if fps_list.len() == 0 {
            stable_fps = 0;
        } else {
            stable_fps = fps_list.iter().sum::<u64>() / fps_list.len() as u64;
        }

        framework.set_title(
            format!(
                "Rust Window {} FPS ({:.2}, {}) | x{} y{} z{} | {} {} -> {} {} -> {} {}",
                stable_fps,
                fps,
                fps_list.len(),
                camera.x,
                camera.y,
                camera.z,
                camera.x + mouse_pos.0 as isize,
                camera.y + mouse_pos.1 as isize,
                mouse_pos.0,
                mouse_pos.1,
                mouse_delta.0,
                mouse_delta.1
            )
            .as_str(),
        );
        // if selected.is_some() {
        //     let block = &blocks[selected.unwrap()];
        //     println!("Block Index: {}, ID: {}, Connected bottom: {}, Connected top: {}, Possible connection below: {}, Possible Connection Above: {}",
        //     selected.unwrap(), block.id,
        //     block.connected_below.get().map_or("None".to_string(), |id| id.to_string()),
        //     block.connected_top.get().map_or("None".to_string(), |id| id.to_string()),
        //     block.possible_connection_below.get().map_or("None".to_string(), |id| id.to_string()),
        //     block.possible_connection_above.get().map_or("None".to_string(), |id| id.to_string()))
        // }

        // WHY THE ACTUAL FUCK IS THE FRAMERATE NOT 60????? I DON'T WANT 1800 FPS WITH 100% CPU
        if delta_time < target_frame_delta && false {
            let sleep_time = target_frame_delta - delta_time;
            // platform::log(&format!(
            //     "Delta: {}, Needed: {}, Sleeping for: {}",
            //     delta_time, target_frame_delta, sleep_time
            // ));
            framework.wait(sleep_time);
        }
        //platform::log("Got through iter")
    }
}

// fn main() {
//     let width = 800;
//     let height = 600;

//     let window_name = "Rust window";
//     // Create a window with the title "Rust Window"
//     let mut window = Window::new(
//         &window_name, // Window title
//         width,        // Width
//         height,       // Height
//         WindowOptions {
//             title: true,
//             ..WindowOptions::default()
//         },
//     )
//     .expect("Unable to create window");

//     #[cfg(target_os = "windows")]
//     window.set_icon(Icon::from_str("src/idk.ico").unwrap());
// }
