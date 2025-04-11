use crate::render::RenderSettings;
use fontdue::Font;

use crate::render::get_glyph_cache;

pub struct RenderSettingsPretty {}

#[inline(always)]
fn round_float_key(value: f32) -> (i32, i32) {
    let multiplier = 10000.0; // Provides 4 decimal places of precision
    let rounded_int_x = (value * multiplier).round() as i32;
    let rounded_int_y = (value * multiplier).fract() as i32;
    (rounded_int_x, rounded_int_y)
}
impl RenderSettingsPretty {
    pub fn new() -> Self {
        return RenderSettingsPretty {};
    }
}

impl RenderSettings for RenderSettingsPretty {
    fn draw_pixel(
        &self,
        buffer: *mut u32,
        width: usize,
        height: usize,
        x: usize,
        y: usize,
        color: u32,
    ) {
        if x >= width || y >= height {
            return;
        }

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

        let rounded_size_key = round_float_key(size);
        let font_metrics = font.horizontal_line_metrics(size).unwrap();

        for ch in text.chars() {
            // Try to get the glyph from cache first
            let (metrics, bitmap) = {
                let cache = get_glyph_cache().read().unwrap();
                cache.get(&(ch, rounded_size_key)).cloned()
            }
            .unwrap_or_else(|| {
                let rasterized = font.rasterize(ch, size);

                // Insert into cache
                let mut cache_mut = get_glyph_cache().write().unwrap();
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
                        + (font_metrics.ascent - metrics.height as f32)
                            as usize;

                    if px < width && py < height {
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

                                self.draw_pixel(
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
    fn adjust_brightness(&self, color: u32, x: i32) -> u32 {
        adjust_brightness(color, x, BrightnessModel::LinearWeighted)
    }
    fn desaturate(&self, color: u32, amount: f32) -> u32 {
        let r = ((color >> 16) & 0xFF) as f32 / 255.0;
        let g = ((color >> 8) & 0xFF) as f32 / 255.0;
        let b = (color & 0xFF) as f32 / 255.0;

        // Convert RGB to HSL
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;

        let s = if max == min {
            0.0
        } else if l < 0.5 {
            (max - min) / (max + min)
        } else {
            (max - min) / (2.0 - max - min)
        };

        // Reduce saturation toward grayscale
        let new_s = s * (1.0 - amount);

        // Reconstruct color from HSL
        let (r2, g2, b2) = hsl_to_rgb_f32(hue_of(r, g, b), new_s, l);
        let r_new = (r2 * 255.0).round().clamp(0.0, 255.0) as u32;
        let g_new = (g2 * 255.0).round().clamp(0.0, 255.0) as u32;
        let b_new = (b2 * 255.0).round().clamp(0.0, 255.0) as u32;

        (r_new << 16) | (g_new << 8) | b_new
    }
}

fn hue_of(r: f32, g: f32, b: f32) -> f32 {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);

    if max == min {
        0.0
    } else if max == r {
        ((g - b) / (max - min)).rem_euclid(6.0) * 60.0
    } else if max == g {
        ((b - r) / (max - min) + 2.0) * 60.0
    } else {
        ((r - g) / (max - min) + 4.0) * 60.0
    }
}

fn adjust_brightness_hsl(color: u32, x: i32) -> u32 {
    // Extract components
    let a = (color >> 24) & 0xFF;
    let r = (color >> 16) & 0xFF;
    let g = (color >> 8) & 0xFF;
    let b = color & 0xFF;

    // Convert to HSL
    let (h, s, l) = rgb_to_hsl(r, g, b);

    // Adjust lightness in HSL space (most perceptually accurate)
    let l_new = (l + x as f32).clamp(0.0, 100.0);

    // Convert back to RGB
    let (r_new, g_new, b_new) = hsl_to_rgb_u32(h, s, l_new);

    // Recombine with alpha
    (a << 24) | (r_new << 16) | (g_new << 8) | b_new
}
fn hsl_to_rgb_f32(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r1, g1, b1) = match h as i32 {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        300..=359 => (c, 0.0, x),
        _ => (0.0, 0.0, 0.0),
    };

    (r1 + m, g1 + m, b1 + m)
}
/// Helper function to convert RGB to HSL color space
/// Returns (hue, saturation, lightness) as (degrees, percentage, percentage)
fn rgb_to_hsl(r: u32, g: u32, b: u32) -> (f32, f32, f32) {
    let r_norm = r as f32 / 255.0;
    let g_norm = g as f32 / 255.0;
    let b_norm = b as f32 / 255.0;

    let max = r_norm.max(g_norm).max(b_norm);
    let min = r_norm.min(g_norm).min(b_norm);
    let delta = max - min;

    // Calculate lightness
    let lightness = (max + min) / 2.0;

    // Calculate saturation
    let saturation = if delta < 0.0001 {
        0.0 // achromatic (gray)
    } else {
        if lightness < 0.5 {
            delta / (max + min)
        } else {
            delta / (2.0 - max - min)
        }
    };

    // Calculate hue
    let hue = if delta < 0.0001 {
        0.0 // achromatic (gray)
    } else if max == r_norm {
        60.0 * (((g_norm - b_norm) / delta) % 6.0)
    } else if max == g_norm {
        60.0 * ((b_norm - r_norm) / delta + 2.0)
    } else {
        60.0 * ((r_norm - g_norm) / delta + 4.0)
    };

    // Return HSL values normalized to standard ranges
    (hue, saturation * 100.0, lightness * 100.0)
}

/// Helper function to convert HSL to RGB color space
/// Takes (hue, saturation, lightness) as (degrees, percentage, percentage)
/// Returns (r, g, b) as u32 values in range 0-255
fn hsl_to_rgb_u32(h: f32, s: f32, l: f32) -> (u32, u32, u32) {
    // Normalize inputs to 0-1 range
    let h_norm = h / 360.0;
    let s_norm = s / 100.0;
    let l_norm = l / 100.0;

    // Handle grayscale case
    if s_norm < 0.0001 {
        let gray = (l_norm * 255.0).round() as u32;
        return (gray, gray, gray);
    }

    // Helper function
    let hue_to_rgb = |p: f32, q: f32, t: f32| -> f32 {
        let t_adj = if t < 0.0 {
            t + 1.0
        } else if t > 1.0 {
            t - 1.0
        } else {
            t
        };

        if t_adj < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t_adj
        } else if t_adj < 1.0 / 2.0 {
            q
        } else if t_adj < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t_adj) * 6.0
        } else {
            p
        }
    };

    // Calculate temp values
    let q = if l_norm < 0.5 {
        l_norm * (1.0 + s_norm)
    } else {
        l_norm + s_norm - l_norm * s_norm
    };
    let p = 2.0 * l_norm - q;

    // Calculate RGB values
    let r = hue_to_rgb(p, q, h_norm + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h_norm);
    let b = hue_to_rgb(p, q, h_norm - 1.0 / 3.0);

    // Convert to 8-bit values
    let r_8bit = (r * 255.0).round() as u32;
    let g_8bit = (g * 255.0).round() as u32;
    let b_8bit = (b * 255.0).round() as u32;

    (r_8bit, g_8bit, b_8bit)
}

/// Higher-level function that provides both perceptual models
enum BrightnessModel {
    LinearWeighted, // Uses RGB with perceptual weights
    HSL,            // Uses HSL color space
}

fn adjust_brightness(color: u32, x: i32, model: BrightnessModel) -> u32 {
    match model {
        BrightnessModel::LinearWeighted => {
            // Extract color components
            let a = (color >> 24) & 0xFF;
            let r = ((color >> 16) & 0xFF) as f32;
            let g = ((color >> 8) & 0xFF) as f32;
            let b = (color & 0xFF) as f32;

            // Apply perceptual weights to adjustment
            let r_adj = x as f32 * 0.2126;
            let g_adj = x as f32 * 0.7152;
            let b_adj = x as f32 * 0.0722;

            // Apply adjustments with clamping
            let r_new = (r + r_adj).clamp(0.0, 255.0) as u32;
            let g_new = (g + g_adj).clamp(0.0, 255.0) as u32;
            let b_new = (b + b_adj).clamp(0.0, 255.0) as u32;

            // Recombine with alpha
            (a << 24) | (r_new << 16) | (g_new << 8) | b_new
        }
        BrightnessModel::HSL => adjust_brightness_hsl(color, x),
    }
}
