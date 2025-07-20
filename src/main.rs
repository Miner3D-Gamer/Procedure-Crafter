#![allow(dead_code)]
#![deny(clippy::needless_return)]
#![allow(clippy::too_many_arguments)]

#![allow(clippy::ptr_arg)]
// Web "works" but nothing is showing up
mod all; // Main loop
mod custom; // Workspace/Blocks/Camera
mod logic; // 'Physics'
use all::main_loop;
mod idk;
use mirl::extensions::*;

//use mirl::platform::framework_traits::Framework;
use mirl::platform::framework_traits::Window;

#[cfg(not(target_arch = "wasm32"))]
fn main() {

    mirl::enable_traceback();
    use mirl::{
        platform::{FileSystem, WindowSettings},
    };

    let buffer = mirl::platform::Buffer::new_empty(1000, 600);
    let mut framework = mirl::platform::minifb::Framework::new(
        "Rust Window",
        *WindowSettings::default(&buffer)
            .set_position(
                mirl::system::info::get_center_of_screen_of_buffer(&buffer)
                    .tuple_2_into(),
            )
            .set_position_to_middle_of_screen(),
    );
    let file_system =
        mirl::platform::file_system::NativeFileSystem::new(Vec::new());

    main_loop(&mut framework, &file_system, &buffer);
}

// #[cfg(target_arch = "wasm32")]
// #[wasm_bindgen::prelude::wasm_bindgen(start)]
// pub fn start() {
//     let renderer = platform::web::WebFramework::new(640, 480);
//     main_loop(renderer);
// }
// #[cfg(target_arch = "wasm32")]
// use wasm_bindgen::prelude::*;

// #[cfg(target_arch = "wasm32")]
// fn main() {}
