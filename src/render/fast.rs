use crate::render::RenderSettings;
pub struct RenderSettingsFast {}

impl RenderSettingsFast {
    pub fn new() -> Self {
        return RenderSettingsFast {};
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

impl RenderSettings for RenderSettingsFast {}

// // Alternative implementation that handles alpha channel
// fn adjust_brightness_with_alpha(color: u32, x: i32) -> u32 {
//     // Extract color components including alpha
//     let a = (color >> 24) & 0xFF;
//     let r = ((color >> 16) & 0xFF) as i32;
//     let g = ((color >> 8) & 0xFF) as i32;
//     let b = (color & 0xFF) as i32;

//     // Calculate new values with clamping
//     let r_new = (r + x).clamp(0, 255) as u32;
//     let g_new = (g + x).clamp(0, 255) as u32;
//     let b_new = (b + x).clamp(0, 255) as u32;

//     // Recombine into a single color value, preserving alpha
//     (a << 24) | (r_new << 16) | (g_new << 8) | b_new
// }
