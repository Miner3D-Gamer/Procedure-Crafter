#![allow(dead_code)]

use core::panic;
use minifb::{Icon, Window, WindowOptions};
use mirl::graphics::rgb_to_u32;

use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::{cell::Cell, str::FromStr};
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
}
struct BlockInput {
    input_type: String,
    name: String,
    expected: Option<Vec<String>>,
    expected_return: Option<Vec<String>>,
}

use fontdue::Font;
mod file_system;

// Drawing
fn draw_text(
    buffer: &mut Vec<u32>,
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

    for ch in text.chars() {
        let (metrics, bitmap) = font.rasterize(ch, size);
        let font_metrics = font.horizontal_line_metrics(size).unwrap();

        // Draw each character into the buffer
        for gy in 0..metrics.height {
            for gx in 0..metrics.width {
                let px = pen_x + gx;
                // Correcting for letter height -> Buggy but working so ¯\_(ツ)_/¯
                let py = pen_y
                    + gy
                    + (font_metrics.ascent - metrics.height as f32) as usize;

                if px < *width && py < *height {
                    let index = py * width + px;
                    let alpha = bitmap[gy * metrics.width + gx]; // Alpha (0-255)

                    if alpha > 0 {
                        let bg = buffer[index];

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
                            / 255) as u8;
                        let ng = ((tg as u16 * alpha as u16
                            + bg as u16 * inv_alpha as u16)
                            / 255) as u8;
                        let nb = ((tb as u16 * alpha as u16
                            + bb as u16 * inv_alpha as u16)
                            / 255) as u8;
                        let na = ((ta as u16 * alpha as u16
                            + ba as u16 * inv_alpha as u16)
                            / 255) as u8;

                        draw_pixel(
                            buffer,
                            &(*width as isize),
                            &(*height as isize),
                            px as isize,
                            py as isize,
                            (nr as u32) << 24
                                | (ng as u32) << 16
                                | (nb as u32) << 8
                                | na as u32,
                        );
                    }
                }
            }
        }

        // Advance the cursor position
        pen_x += metrics.advance_width as usize;
    }
}

fn draw_pixel(
    buffer: &mut Vec<u32>,
    width: &isize,
    height: &isize,
    x: isize,
    y: isize,
    color: u32,
) {
    if FAST_RENDER {
        draw_pixel_unsafe(buffer, width, height, x, y, color);
    } else {
        draw_pixel_safe(buffer, width, height, x, y, color);
    }
}

fn draw_pixel_safe(
    buffer: &mut Vec<u32>,
    width: &isize,
    height: &isize,
    x: isize,
    y: isize,
    color: u32,
) {
    if x < 0 || y < 0 {
        return;
    }
    if x >= *width || y >= *height {
        return;
    }
    draw_pixel_unsafe(buffer, width, height, x, y, color);
}
fn draw_pixel_unsafe(
    buffer: &mut Vec<u32>,
    width: &isize,
    height: &isize,
    x: isize,
    y: isize,
    color: u32,
) {
    let index = y * width + x;
    if false {
        let found = get_pixel(buffer, width, height, x, y);
        if found != 0 {
            return;
        }
    }
    buffer[index as usize] = color; //mirl::graphics::rgb_to_u32(255, 0, 0);
}
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
    buffer: &mut Vec<u32>,
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
        width,
        height,
        font,
    );
}

fn render_ghost_block(
    buffer: &mut Vec<u32>,
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
        );
        // get_block_id_above(
        //     blocks,
        //     blocks[selected.unwrap()].x.get() as f32,
        //     blocks[selected.unwrap()].y.get() as f32,
        // );
        if possible.is_some() {
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
                &(*width as isize),
                &(*height as isize),
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
    buffer: &mut Vec<u32>,
    block_color: u32,
    width: &isize,
    height: &isize,
    font: &Font,
) {
    for x in (origin_x - camera.x)
        ..(origin_x + block.width.get() as isize - camera.x)
    {
        for y in (origin_y - camera.y)
            ..(origin_y + block.height.get() as isize - camera.y)
        {
            {
                draw_pixel(buffer, &width, &height, x, y, block_color);
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
    buffer: &mut Vec<u32>,
    width: &usize,
    height: &usize,
    camera: &Camera,
    blocks: &Vec<Block>,
    block_colors: &Vec<u32>,
    font: &Font,
) {
    let now_width = *width as isize;
    let now_height = *height as isize;
    // Reverse block order in order for overdraw to to its job in our favor
    for block in blocks.iter().rev() {
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
) -> Option<usize> {
    if DIRTY_LOGIC {
        return get_any_block_in_distance(
            blocks,
            pos_x,
            pos_y,
            max_distance,
            blacklisted,
        );
    } else {
        get_closest_block_in_distance(
            blocks,
            pos_x,
            pos_y,
            max_distance,
            blacklisted,
        )
    }
}

fn get_closest_block_in_distance(
    blocks: &[Block],
    pos_x: f32,
    pos_y: f32,
    max_distance: f32,
    blacklisted: Option<usize>,
) -> Option<usize> {
    let mut closest = None;
    let mut min_distance = max_distance; // Start with max distance as the limit

    for (block_id, block) in blocks.iter().enumerate() {
        if blacklisted.is_some() {
            if block_id == blacklisted.unwrap() {
                continue;
            }
        }
        let distance = get_distance_between_positions(
            pos_x,
            pos_y,
            block.x.get() as f32,
            block.y.get() as f32,
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
) -> Option<usize> {
    for block_id in 0..blocks.len() {
        if blacklisted.is_some() {
            if block_id == blacklisted.unwrap() {
                continue;
            }
        }
        let block = &blocks[block_id];
        if block.block_type == 0 {
            if get_distance_between_positions(
                pos_x,
                pos_y,
                block.x.get() as f32,
                block.y.get() as f32,
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

fn new_block(
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
    }
}
fn get_distance_between_positions(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    if DIRTY_LOGIC {
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
const FAST_RENDER: bool = false;
const DIRTY_LOGIC: bool = false;
static GLOBAL_BLOCK_COUNTER: Lazy<Mutex<usize>> = Lazy::new(|| Mutex::new(0));

fn main() {
    let width = 800;
    let height = 600;
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

    let mut frame_start;

    let title_bat_height = mirl::system::get_title_bar_height();
    let (screen_width, screen_height) = mirl::system::get_screen_resolution();

    let mut delta_time;
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
    let snap_distance = 70.0;
    let scroll_multiplier = 5.0;

    let mut blocks: Vec<Block> = Vec::new();

    //let mut color_names: Vec<String> = Vec::new();
    let mut color_rgb: Vec<u32> = Vec::new();
    let mut color_names: Vec<String> = Vec::new();

    //color_names.push("bool".to_string());
    color_rgb.push(rgb_to_u32(50, 80, 255));
    color_names.push("bool".to_string());
    let font = file_system::load_font("src/inter.ttf");

    for _ in 0..100 {
        blocks.push(new_block(
            "new block".to_string(),
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

    let mut selected: Option<usize> = None;

    let mut mouse_outside;

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
            } else {
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
                if window.is_key_down(minifb::Key::LeftCtrl) {
                    camera.z -= mouse_wheel_temp.unwrap().1;
                } else {
                    if window.is_key_down(minifb::Key::LeftShift) {
                        camera.y -= (mouse_wheel_temp.unwrap().0
                            * scroll_multiplier)
                            as isize;
                        camera.x -= (mouse_wheel_temp.unwrap().1
                            * scroll_multiplier)
                            as isize;
                    } else {
                        camera.x -= (mouse_wheel_temp.unwrap().0
                            * scroll_multiplier)
                            as isize;
                        camera.y -= (mouse_wheel_temp.unwrap().1
                            * scroll_multiplier)
                            as isize;
                    }
                }
            }
        }

        //############################################

        handle_and_render_on_screen(
            &mut buffer,
            &width,
            &height,
            &camera,
            &blocks,
            &color_rgb,
            &font,
        );
        render_ghost_block(
            &mut buffer,
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

        delta_time = mirl::time::get_elapsed_as_us(frame_start);

        if delta_time != 0 {
            fps = 1_000_000 / delta_time; // Convert nanoseconds to FPS
        } else {
            fps = u128::MAX;
        }

        //println!("FPS: {}", fps);
        window.set_title(
            format!(
                "Rust Window {} FPS | x{} y{} z{} | {} {} -> {} {}",
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
    }
}
