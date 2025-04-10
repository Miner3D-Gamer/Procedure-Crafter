use crate::custom::Block;
use crate::custom::Camera;
use crate::logic::Physics;
use fontdue::Font;

use fontdue::Metrics;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;

static GLYPH_CACHE: Lazy<
    RwLock<HashMap<(char, (i32, i32)), (Metrics, Vec<u8>)>>,
> = Lazy::new(|| RwLock::new(HashMap::new()));

#[inline(always)]
fn get_glyph_cache(
) -> &'static RwLock<HashMap<(char, (i32, i32)), (Metrics, Vec<u8>)>> {
    &GLYPH_CACHE
}

#[inline(always)]
fn round_float_key(value: f32) -> (i32, i32) {
    let multiplier = 10.0_f32.powi(4);
    let rounded_int_x = (value * multiplier).round() as i32;
    let rounded_int_y = (value * multiplier).fract() as i32;
    (rounded_int_x, rounded_int_y)
}

pub trait RenderSettings {
    fn draw_pixel(
        &self,
        buffer: *mut u32,
        width: usize,
        _height: usize,
        x: usize,
        y: usize,
        color: u32,
    ) {
        unsafe {
            *buffer.add(y * width + x) = color;
        }
    }
    fn draw_text(
        &self,
        buffer: *mut u32,
        width: usize,
        height: usize,
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
                let cache = get_glyph_cache().read().unwrap();
                cache.get(&(ch, rounded_size_key)).cloned()
            };

            // If not in cache, rasterize and insert
            let (metrics, bitmap) = cached_glyph.unwrap_or_else(|| {
                let rasterized = font.rasterize(ch, size);

                // Insert into cache
                let mut cache_mut = get_glyph_cache().write().unwrap();
                cache_mut.insert((ch, rounded_size_key), rasterized.clone());

                rasterized
            });

            let offset_y = ascent.saturating_sub(metrics.height);
            let w = metrics.width;
            let h = metrics.height;
            let advance_x = metrics.advance_width as usize;

            for gy in 0..h {
                let py = pen_y + gy + offset_y;
                if py >= height {
                    continue;
                }

                let row_start = gy * w;
                for gx in 0..w {
                    let px = pen_x + gx;
                    if px >= width {
                        continue;
                    }

                    if bitmap[row_start + gx] > 0 {
                        self.draw_pixel(buffer, width, height, px, py, color);
                    }
                }
            }
            pen_x += advance_x;
        }
    }
    fn render_block<L: Physics>(
        &self,
        block: &Block,
        origin_x: isize,
        origin_y: isize,
        camera: &Camera,
        buffer: *mut u32,
        block_color: u32,
        width: usize,
        height: usize,
        font: &Font,
        logic: &L,
    ) {
        let x0 = block.x.get() as isize;
        let y0 = block.y.get() as isize;
        let x1 = block.x.get() as isize + block.width.get() as isize;
        let y1 = block.y.get() as isize + block.height.get() as isize;
        let holes = &[(1, 1, 40, 20)];

        for y in y0..y1 {
            for x in x0..x1 {
                if logic.is_in_any_hole(x - x0, y - y0, holes) {
                    self.draw_pixel(
                        buffer,
                        width,
                        height,
                        (x - camera.x) as usize,
                        (y - camera.y) as usize,
                        self.adjust_brightness(block_color, 50),
                    );
                    continue;
                }

                self.draw_pixel(
                    buffer,
                    width,
                    height,
                    (x - camera.x) as usize,
                    (y - camera.y) as usize,
                    block_color,
                );
            }
        }

        self.draw_text(
            buffer,
            width,
            height,
            &block.name,
            (origin_x - camera.x) as usize,
            (origin_y - camera.y) as usize,
            mirl::graphics::rgb_to_u32(255, 0, 0),
            20.0,
            font,
        );
    }
    fn draw_circle(
        &self,
        buffer: *mut u32,
        width: usize,
        height: usize,
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
            self.draw_pixel(
                buffer,
                width,
                height,
                pos_x + temp_x,
                pos_y + temp_y,
                color,
            );
            self.draw_pixel(
                buffer,
                width,
                height,
                pos_x - temp_x,
                pos_y + temp_y,
                color,
            );
            self.draw_pixel(
                buffer,
                width,
                height,
                pos_x + temp_x,
                pos_y - temp_y,
                color,
            );
            self.draw_pixel(
                buffer,
                width,
                height,
                pos_x - temp_x,
                pos_y - temp_y,
                color,
            );
            self.draw_pixel(
                buffer,
                width,
                height,
                pos_x + temp_y,
                pos_y + temp_x,
                color,
            );
            self.draw_pixel(
                buffer,
                width,
                height,
                pos_x + temp_y,
                pos_y - temp_x,
                color,
            );
            self.draw_pixel(
                buffer,
                width,
                height,
                pos_x - temp_y,
                pos_y + temp_x,
                color,
            );
            self.draw_pixel(
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
    fn adjust_brightness(&self, color: u32, x: i32) -> u32 {
        // Extract color components
        let r = ((color >> 16) & 0xFF) as i32;
        let g = ((color >> 8) & 0xFF) as i32;
        let b = (color & 0xFF) as i32;

        // Calculate new values with clamping
        let r_new = (r + x).clamp(0, 255) as u32;
        let g_new = (g + x).clamp(0, 255) as u32;
        let b_new = (b + x).clamp(0, 255) as u32;

        // Recombine into a single color value
        (r_new << 16) | (g_new << 8) | b_new
    }
}

//#[cfg(not(feature = "fast_render"))]
pub mod pretty;
//#[cfg(not(feature = "fast_render"))]
pub use pretty::RenderSettingsPretty;

// //#[cfg(feature = "fast_render")]
// pub mod fast;
// //#[cfg(feature = "fast_render")]
// pub use fast::RenderSettingsFast;
