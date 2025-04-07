#![allow(dead_code)]
// Web "works" but nothing is showing up
mod all;
mod custom_id;
mod platform;
use all::main_loop;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let renderer = platform::native::NativeFramework::new(800, 600, "TEST");
    let file_system = platform::native::NativeFileSystem::new();
    main_loop(renderer, file_system);
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
