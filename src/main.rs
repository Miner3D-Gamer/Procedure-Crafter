#![allow(dead_code)]
#![allow(static_mut_refs)] // Yeah, this is probably fine

use core::panic;
use minifb::{Icon, Window, WindowOptions};
use mirl::graphics::rgb_to_u32;

use fontdue::Font;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::{cell::Cell, str::FromStr};
mod file_system;
// Cell: Make attribute mutable while keeping the rest of the struct static

struct Camera {
    x: isize,
    y: isize,
    z: f32,
}

struct Block {
    name: String,
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
    id: usize,
    connected_top: Cell<Option<usize>>,
    connected_below: Cell<Option<usize>>,
    possible_connection_above: Cell<Option<usize>>,
    possible_connection_below: Cell<Option<usize>>,
    recently_moved: Cell<bool>,
}

impl Block {
    fn new(
        name: String,
        x: i16,
        y: i16,
        block_type: u8,
        required_imports: Vec<String>,
        required_contexts: Vec<String>,
        file_versions: Vec<String>,
        file_locations: Vec<String>,
        output: String,
        inputs: Vec<BlockInput>,
        color_types: &Vec<String>,
        font: &Font,
    ) -> Block {
        let color_id = color_types.iter().position(|x| *x == output).unwrap();
        let x = ((u16::MAX / 2) as i16 + x) as u16;
        let y = ((u16::MAX / 2) as i16 + y) as u16;

        Block {
            name: name.clone(),
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
            id: increment_global_block_id(),
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
use std::collections::HashMap;
use std::sync::Once;

static INIT: Once = Once::new();
static mut GLYPH_CACHE: Option<
    RefCell<HashMap<(char, (i32, i32)), (Metrics, Vec<u8>)>>,
> = None;

fn get_glyph_cache(
) -> &'static RefCell<HashMap<(char, (i32, i32)), (Metrics, Vec<u8>)>> {
    unsafe {
        INIT.call_once(|| {
            GLYPH_CACHE = Some(RefCell::new(HashMap::new()));
        });
        GLYPH_CACHE.as_ref().unwrap()
    }
}
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
                            let inv_alpha = 255 - alpha;
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
fn get_top_most_block_id_or_self(blocks: &Vec<Block>, id: usize) -> usize {
    let mut block = &blocks[id];
    while block.possible_connection_above.get().is_some() {
        block = &blocks[block.possible_connection_above.get().unwrap()];
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

        if possible.is_some() {
            if is_there_a_loop_in_block_connections_for_block_internal(
                &blocks,
                blocks[possible.unwrap()].id,
                &mut Vec::from([blocks[selected.unwrap()].id]),
            ) {
                blocks[selected.unwrap()].possible_connection_above.set(None);
                return;
            } else {
                // Save block, only if it is not a loop
                blocks[selected.unwrap()]
                    .possible_connection_above
                    .set(possible);
            }
            let above_block = &blocks[possible.unwrap()];

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
                rgb_to_u32(100, 100, 100),
                width,
                height,
                font,
            );
        }
    }
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
    for x in (origin_x - camera.x)
        ..(origin_x + block.width.get() as isize - camera.x)
    {
        for y in (origin_y - camera.y)
            ..(origin_y + block.height.get() as isize - camera.y)
        {
            {
                draw_pixel(
                    buffer,
                    width,
                    height,
                    x as usize,
                    y as usize,
                    block_color,
                );
            }
        }
    }

    draw_text(
        buffer,
        &(*width as usize),
        &(*height as usize),
        &block.name,
        (origin_x - camera.x) as usize,
        (origin_y - camera.y) as usize,
        rgb_to_u32(255, 0, 0),
        20.0,
        font,
    );
}

fn is_block_visible_on_screen(
    block: &Block,
    camera: &Camera,
    width: &isize,
    height: &isize,
) -> bool {
    if FAST_RENDER {
        return is_block_fully_visible_on_screen(block, camera, width, height);
    } else {
        return is_block_visible_on_screen_proper(block, camera, width, height);
    }
}

// Misc
fn is_block_visible_on_screen_proper(
    block: &Block,
    camera: &Camera,
    width: &isize,
    height: &isize,
) -> bool {
    // check if any corner is visible
    let cam_x = camera.x as f32;
    let cam_y = camera.y as f32;
    let cam_width = *width as f32;
    let cam_height = *height as f32;
    let x = block.x.get() as f32;
    let y = block.y.get() as f32;
    let x2 = x + block.width.get();
    let y2 = y + block.height.get();

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

fn is_block_fully_visible_on_screen(
    block: &Block,
    camera: &Camera,
    width: &isize,
    height: &isize,
) -> bool {
    // Check if any corner isn't visible
    let cam_x = camera.x as f32;
    let cam_y = camera.y as f32;
    let cam_width = *width as f32;
    let cam_height = *height as f32;
    let x = block.x.get() as f32;
    let y = block.y.get() as f32;
    let x2 = x + block.width.get();
    let y2 = y + block.height.get();

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
        move_block_to_connected(blocks, &Some(id));
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
fn subtract_tuple(one: (f32, f32), two: (f32, f32)) -> (f32, f32) {
    (one.0 - two.0, one.1 - two.1)
}
fn reorder_element<T>(vec: &mut Vec<T>, from: usize, to: usize) {
    if from != to && from < vec.len() && to < vec.len() {
        let item = vec.remove(from);
        vec.insert(to, item)
    }
}
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
fn get_global_block_id() -> usize {
    return GLOBAL_BLOCK_COUNTER.lock().unwrap().clone();
}
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

fn get_distance_between_positions(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    if FAST_LOGIC {
        return get_approx_distance(x1, y1, x2, y2);
    } else {
        return get_distance_between_positions_accurate(x1, y1, x2, y2);
    }
}
fn get_distance_between_positions_accurate(
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
) -> f32 {
    return ((x1 - x2) * (x1 - x2) + (y1 - y2) * (y1 - y2)).sqrt();
}
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

fn index_by_block_id(id: usize, blocks: &Vec<Block>) -> Option<usize> {
    for block_id in 0..blocks.len() {
        if blocks[block_id].id == id {
            return Some(block_id);
        }
    }
    return None;
}

fn is_there_a_loop_in_block_connections_for_block(
    blocks: &Vec<Block>,
    block_id: usize,
) -> bool {
    let mut already_checked = Vec::new();
    return is_there_a_loop_in_block_connections_for_block_internal(
        blocks,
        block_id,
        &mut already_checked,
    );
}
fn is_there_a_loop_in_block_connections_for_block_internal(
    blocks: &Vec<Block>,
    block_index: usize,
    already_checked: &mut Vec<usize>,
) -> bool {
    if already_checked.contains(&block_index) {
        return true;
    }
    already_checked.push(block_index);
    let block = &blocks[index_by_block_id(block_index, blocks).unwrap()];
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
fn get_ids_connected_to_block(top: usize, blocks: &Vec<Block>) -> Vec<usize> {
    let block = &blocks[index_by_block_id(top, &blocks).unwrap()];
    let mut list = Vec::new();
    list.push(top);
    if block.connected_below.get().is_some() {
        list.extend(get_ids_connected_to_block(
            block.connected_below.get().unwrap(),
            blocks,
        ))
    }
    return list;
}
fn index_by_block_ids(
    blocks: &Vec<Block>,
    ids: Vec<usize>,
) -> Vec<Option<usize>> {
    let mut return_list = Vec::new();
    for id in ids {
        return_list.push(index_by_block_id(id, blocks))
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

fn move_block_to_connected(blocks: &mut Vec<Block>, selected: &Option<usize>) {
    if selected.is_none() {
        return;
    }
    let block = &blocks[selected.unwrap()];
    let top_block_id = block.connected_top.get();
    if top_block_id.is_none() {
        return;
    }
    let top_block =
        &blocks[index_by_block_id(top_block_id.unwrap(), blocks).unwrap()];
    let block_query = get_ids_connected_to_block(top_block_id.unwrap(), blocks);
    let total_offset = get_total_height_of_blocks(
        blocks,
        index_by_block_ids(blocks, block_query),
    );
    println!("{total_offset} {}", total_offset as u16 + top_block.y.get());

    block.x.set(top_block.x.get());
    block.y.set(total_offset as u16 + top_block.y.get());
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

// Goal: Render 1000 Blocks at >=60 fps
// Current: 10000 at ~40 fps
// ------------------------------------------< SETTINGS >------------------------------------------

const FAST_RENDER: bool = false; // ~10% <-> ~40% Faster rendering
const FAST_LOGIC: bool = false; // No analytics

// ------------------------------------------------------------------------------------------------

static GLOBAL_BLOCK_COUNTER: Lazy<Mutex<usize>> = Lazy::new(|| Mutex::new(0));

fn main() {
    let width = 800;
    let height = 600;

    let snap_distance = 70.0;
    let scroll_multiplier = 5.0;
    let max_fps = 30;

    let window_name = "Rust window";
    // Create a window with the title "Rust Window"
    let mut window = Window::new(
        &window_name, // Window title
        width,        // Width
        height,       // Height
        WindowOptions {
            title: true,
            ..WindowOptions::default()
        },
    )
    .expect("Unable to create window");

    #[cfg(target_os = "windows")]
    window.set_icon(Icon::from_str("src/cot.ico").unwrap());

    let target_frame_delta = mirl::time::MICROS_PER_SEC / max_fps; // Time for one frame at the target FPS
    let mut frame_start;

    let title_bat_height = mirl::system::get_title_bar_height();
    let (screen_width, screen_height) = mirl::system::get_screen_resolution();

    let mut delta_time: u64;
    let mut buffer: Vec<u32>;
    let mut fps;

    let mut mouse_pos = window.get_mouse_pos(minifb::MouseMode::Pass).unwrap();
    let mut mouse_delta;

    let mut mouse_wheel_temp;

    let mut camera = Camera {
        x: (u16::MAX / 2) as isize,
        y: (u16::MAX / 2) as isize,
        z: 1.0,
    };

    let mut blocks: Vec<Block> = Vec::new();

    //let mut color_names: Vec<String> = Vec::new();
    let mut color_rgb: Vec<u32> = Vec::new();
    let mut color_names: Vec<String> = Vec::new();

    //color_names.push("bool".to_string());
    color_rgb.push(rgb_to_u32(50, 80, 255));
    color_names.push("bool".to_string());
    let font = file_system::load_font("src/inter.ttf");

    for id in 0..10 {
        blocks.push(Block::new(
            format!("new block {}", id + 1),
            rand::random::<i16>() / 100,
            rand::random::<i16>() / 100,
            0,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            "bool".to_string(),
            Vec::new(),
            &color_names,
            &font,
        ))
    }

    let mut last_mouse_down;
    let mut mouse_down_temp;
    let mut mouse_down = false;

    let mut fps_list: Vec<u64> = Vec::new();

    let mut selected: Option<usize> = None;

    let mut mouse_outside;
    let mut stable_fps: u64;

    // Set window to be dead centered
    window.set_position(
        screen_width as isize / 2 - width as isize / 2,
        screen_height as isize / 2
            - height as isize / 2
            - title_bat_height as isize,
    );
    while window.is_open() {
        frame_start = mirl::time::get_time();
        buffer = mirl::render::clear_screen(width, height);

        let buffer_pointer: *mut u32 = buffer.as_mut_ptr();

        // if (buffer_pointer as usize) % 16 == 0 {
        //     println!("Buffer is 16-byte aligned");
        // } else {
        //     println!("Buffer is NOT 16-byte aligned");
        // }
        // Mouse stuff and block(/camera) selection/movement
        mouse_delta = mouse_pos;
        mouse_pos = window.get_mouse_pos(minifb::MouseMode::Pass).unwrap();

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
        mouse_down = window.get_mouse_down(minifb::MouseButton::Left);
        last_mouse_down = mouse_down_temp && mouse_down;

        // There are too many problems with dealing with null when the mouse is outside the window, so instead we just check if the mouse is with in the window :)
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
                    blocks[selected.unwrap()].connected_top.set(None);
                    let list = get_ids_connected_to_block(
                        blocks[selected.unwrap()].id,
                        &blocks,
                    );
                    // find position of element with value blocks[selected.unwrap()].id
                    let split_point = list.iter().position(|block| {
                        *block == blocks[selected.unwrap()].id
                    });
                    split_point.unwrap();
                    if split_point.unwrap() > 0 {
                        // Remove the connected tag from the block this block is connected to (But only if that block above exists!)
                        let block_above = &blocks[index_by_block_id(
                            list[split_point.unwrap() - 1],
                            &blocks,
                        )
                        .unwrap()];
                        block_above.connected_below.set(None);
                    }

                    // Move all blocks connected to the selected block
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
                                    blocks[selected.unwrap()]
                                        .possible_connection_above
                                        .get()
                                        .unwrap(),
                                ),
                                &mut Vec::from([blocks[selected.unwrap()].id]),
                            ) {
                                panic!("Loop detected");
                            }
                        // Tell current block to connect above
                        block.connected_top.set(Some(
                            get_top_most_block_id_or_self(
                                &blocks,
                                connection_id_above,
                            ),
                        ));
                        // Get above block
                        let above_block = &blocks[index_by_block_id(
                            connection_id_above,
                            &blocks,
                        )
                        .unwrap()];
                        above_block.connected_below.set(Some(block.id));
                        // Set current possible above block to none
                        block.possible_connection_above.set(None)
                    }
                }
                selected = None;
            }

            if window.get_mouse_down(minifb::MouseButton::Left) {
                if let Some(mut idx) = selected {
                    reorder_element(&mut blocks, idx, 0);
                    idx = 0;
                    selected = Some(0);
                    blocks[idx].x.set(
                        (blocks[idx].x.get() as f32 + mouse_delta.0) as u16,
                    );
                    blocks[idx].y.set(
                        (blocks[idx].y.get() as f32 + mouse_delta.1) as u16,
                    );
                } else {
                    camera.x -= mouse_delta.0 as isize;
                    camera.y -= mouse_delta.1 as isize;
                }
            }

            // Mouse wheel movement
            mouse_wheel_temp = window.get_scroll_wheel();
            if mouse_wheel_temp.is_some() {
                let extra_mul;
                if window.is_key_down(minifb::Key::RightShift) {
                    extra_mul = 10.0;
                } else {
                    extra_mul = 1.0;
                }
                if window.is_key_down(minifb::Key::LeftCtrl) {
                    camera.z -= mouse_wheel_temp.unwrap().1 * extra_mul;
                } else {
                    if window.is_key_down(minifb::Key::LeftShift) {
                        camera.y -= (mouse_wheel_temp.unwrap().0
                            * scroll_multiplier
                            * extra_mul)
                            as isize;
                        camera.x -= (mouse_wheel_temp.unwrap().1
                            * scroll_multiplier
                            * extra_mul)
                            as isize;
                    } else {
                        camera.x -= (mouse_wheel_temp.unwrap().0
                            * scroll_multiplier
                            * extra_mul)
                            as isize;
                        camera.y -= (mouse_wheel_temp.unwrap().1
                            * scroll_multiplier
                            * extra_mul)
                            as isize;
                    }
                }
            }
        }

        //############################################

        handle_and_render_on_screen(
            buffer_pointer,
            &width,
            &height,
            &camera,
            &mut blocks,
            &color_rgb,
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

        //############################################
        window
            .update_with_buffer(&buffer, width, height)
            .expect("Unable to update window :(");

        delta_time = mirl::time::get_elapsed_as_us(frame_start) as u64;

        if delta_time != 0 {
            fps = mirl::time::MICROS_PER_SEC / delta_time;
        } else {
            fps = u64::MAX;
        }

        add_item_to_max_sized_list(&mut fps_list, fps as usize, fps);
        if fps_list.len() == 0 {
            stable_fps = 0;
        } else {
            stable_fps = fps_list.iter().sum::<u64>() / fps_list.len() as u64;
        }
        //println!("FPS: {}", fps);
        window.set_title(
            format!(
                "Rust Window {} FPS ({}) | x{} y{} z{} | {} {} -> {} {}",
                stable_fps,
                fps,
                camera.x,
                camera.y,
                camera.z,
                mouse_pos.0,
                mouse_pos.1,
                mouse_delta.0,
                mouse_delta.1
            )
            .as_str(),
        );

        if delta_time < target_frame_delta {
            let sleep_time: u64 = target_frame_delta - delta_time;
            std::thread::sleep(std::time::Duration::from_micros(sleep_time));
        }
    }
}
