use crate::platform::shared::{Framework, KeyCode, MouseButton};
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}

pub struct WebFramework {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    window: web_sys::Window,
    width: usize,
    height: usize,
}
use wasm_bindgen::Clamped;
//#[wasm_bindgen]
impl WebFramework {
    pub fn new(width: usize, height: usize) -> Self {
        log("Hi");
        // Get the window and document using web_sys::window
        let window = web_sys::window().expect("Failed to get window");
        let document = window.document().expect("Couldn't get document");

        // Create the canvas element
        let canvas = document
            .create_element("canvas")
            .expect("Couldn't create canvas")
            .dyn_into::<HtmlCanvasElement>()
            .expect("Couldn't cast to canvas");
        // Set canvas size
        canvas.set_width(width as u32);
        canvas.set_height(height as u32);

        // Append the canvas to the body or a specific element in the DOM
        let body = document.body().expect("Couldn't get body");
        body.append_child(&canvas).expect("Couldn't append canvas");

        // Get the 2D rendering context
        let context = canvas
            .get_context("2d")
            .expect("Couldn't get 2d context")
            .expect("Couldn't get 2d context²")
            .dyn_into::<CanvasRenderingContext2d>()
            .expect("Couldn't cast to 2d context³");

        // Initialize the WebFramework struct
        WebFramework {
            canvas,
            context,
            window,
            width,
            height,
        }
    }

    // Method to render a simple shape (for testing)
    pub fn render(&self) {
        self.context.set_fill_style(&"blue".into());
        self.context.fill_rect(10.0, 10.0, 100.0, 100.0);
    }

    pub fn _render(&self, buffer: &[u32]) {
        // Create a Vec<u8> to hold the RGBA values
        let mut rgba_bytes = Vec::with_capacity(buffer.len() * 4);

        for &pixel in buffer {
            // Extract RGBA components from the u32 value
            rgba_bytes.push(((pixel >> 24) & 0xFF) as u8); // Red
            rgba_bytes.push(((pixel >> 16) & 0xFF) as u8); // Green
            rgba_bytes.push(((pixel >> 8) & 0xFF) as u8); // Blue
            rgba_bytes.push((pixel & 0xFF) as u8); // Alpha
        }

        // Wrap the slice in Clamped
        let clamped_data = Clamped(&rgba_bytes[..]);

        // Create the ImageData using the Clamped data
        let image_data = ImageData::new_with_u8_clamped_array_and_sh(
            clamped_data,
            self.width as u32,
            self.height as u32,
        )
        .unwrap();

        // Put the image data on the canvas
        self.context.put_image_data(&image_data, 0.0, 0.0).unwrap();
    }
}
// We need to save the entire directory into the wasm file
impl Framework for WebFramework {
    fn update(&mut self, buffer: &[u32]) {
        self._render(buffer);
    }

    fn is_open(&self) -> bool {
        true // or manage exit flag manually
    }
    fn get_mouse_position(&self) -> Option<(f32, f32)> {
        None
    }
    fn is_key_down(&self, key: KeyCode) -> bool {
        false
    }
    fn get_size(&self) -> (usize, usize) {
        (self.width, self.height)
    }
    fn is_mouse_down(&self, button: MouseButton) -> bool {
        false
    }
    fn get_mouse_scroll(&self) -> Option<(f32, f32)> {
        Some((0.0, 0.0))
    }
    fn set_title(&mut self, title: &str) {}
    fn get_time(&self) -> f64 {
        let performance =
            self.window.performance().expect("no performance exists");
        performance.now() // Returns time in milliseconds with high precision.
    }
    fn get_elapsed_time(&self, start: f64) -> f64 {
        self.get_time() - start
    }
    fn wait(&self, time: f64) {
        // Convert microseconds to milliseconds for setTimeout
        let sleep_time_ms = time / 1000.0;

        // Call JavaScript's setTimeout to simulate sleep
        let closure = Closure::wrap(Box::new(move || {
            // This closure runs after the specified sleep time
            //web_sys::console::log_1(&"WASM sleep finished.".into());
        }) as Box<dyn Fn()>);

        // Set a timeout to simulate sleep (non-blocking)
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                sleep_time_ms as i32,
            )
            .unwrap();

        // Forget the closure so it can be called later
        closure.forget();
    }
}
//use js_sys::Uint8Array;

//use wasm_bindgen_futures::JsFuture;
//use web_sys::window;

pub fn load_font(_: &str) -> fontdue::Font {
    fontdue::Font::from_bytes(
        &include_bytes!("../../src/inter.ttf")[..],
        fontdue::FontSettings::default(),
    )
    .expect("Failed to load font")
}
// pub async fn load_font() -> Font<'static> {
//     let response =
//         JsFuture::from(window().unwrap().fetch_with_str("inter.ttf"))
//             .await
//             .unwrap()
//             .dyn_into::<web_sys::Response>()
//             .unwrap();

//     let buffer =
//         JsFuture::from(response.array_buffer().unwrap()).await.unwrap();
//     let bytes = Uint8Array::new(&buffer).to_vec();

//     Font::try_from_vec(bytes).expect("Failed to parse font")
// }
