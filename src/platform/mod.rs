#[cfg(not(target_arch = "wasm32"))]
pub mod native;
#[cfg(target_arch = "wasm32")]
pub mod web;

// Import everything from the correct module
#[cfg(not(target_arch = "wasm32"))]
pub use native::*;
#[cfg(target_arch = "wasm32")]
pub use web::*;

pub mod shared;
#[allow(unused_imports)]
pub use shared::{KeyCode, MouseButton};
