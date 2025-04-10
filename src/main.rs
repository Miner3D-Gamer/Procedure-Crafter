#![allow(dead_code)]
#![deny(clippy::needless_return)]
// Web "works" but nothing is showing up
mod all;
mod custom;
mod idk;
mod logic;
mod platform;
mod render;
use all::main_loop;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let mut renderer = platform::native::NativeFramework::new(800, 600, "TEST");
    let file_system = platform::native::NativeFileSystem::new();
    let render_settings = render::RenderSettingsPretty::new();
    let logic = logic::Logic::new();
    main_loop(&mut renderer, &file_system, &render_settings, &logic);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    let renderer = platform::web::WebFramework::new(640, 480);
    main_loop(renderer);
}
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
fn main() {}
