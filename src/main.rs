#![allow(dead_code)]
#![deny(clippy::needless_return)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::ptr_arg)]
#![allow(clippy::type_complexity)]

mod all; // Main loop
mod custom; // Workspace/Blocks/Camera
mod logic; // 'Physics'
use all::main_loop;
mod idk;
use mirl::platform::{FileSystem, WindowSettings};

//use mirl::platform::framework_traits::Framework;
use mirl::platform::framework_traits::Window;

// #[cfg(not(target_arch = "wasm32"))]

fn main() {
    mirl::enable_traceback();
    const MB: usize = 1024 * 1024;
    let child = std::thread::Builder::new()
        .stack_size(64 * MB)
        .name("⛲ ⛲ ⛲".to_string())
        .spawn(|| {
            actual_main();
        })
        .unwrap();

    let result = child.join();
    match result {
        Ok(_) => {}
        Err(_) => {
            println!("Not gud")
        }
    }
}
fn actual_main() {
    let buffer = mirl::platform::Buffer::new_empty(1000, 600);
    let mut framework = mirl::platform::minifb::Framework::new(
        "Rust Window",
        *WindowSettings::default(&buffer).set_position_to_middle_of_screen(),
    );
    let file_system =
        mirl::platform::file_system::NativeFileSystem::new(Vec::from([
            "inter.ttf",
        ]))
        .unwrap();
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
