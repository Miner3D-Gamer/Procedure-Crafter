use std::{str::FromStr, time::Duration};

use crate::platform::shared::{Framework, KeyCode, MouseButton};
use minifb::{Window, WindowOptions};

use super::{
    file_data::FileData,
    shared::{FileSystem, Time},
};

pub fn log(s: &str) {
    println!("{}", s);
}
pub struct NativeFramework {
    window: Window,
}

impl NativeFramework {
    pub fn new(width: usize, height: usize, title: &str) -> Self {
        let mut window =
            Window::new(title, width, height, WindowOptions::default())
                .unwrap();

        let title_bat_height = mirl::system::get_title_bar_height();
        let (screen_width, screen_height) =
            mirl::system::get_screen_resolution();

        // Set window to be dead centered
        window.set_position(
            screen_width as isize / 2 - width as isize / 2,
            screen_height as isize / 2
                - height as isize / 2
                - title_bat_height as isize,
        );

        Self {
            window,
        }
    }
}

impl Framework for NativeFramework {
    #[inline]
    fn update(&mut self, buffer: &[u32]) {
        self.window
            .update_with_buffer(
                buffer,
                self.window.get_size().0,
                self.window.get_size().1,
            )
            .unwrap();
    }

    #[inline]
    fn is_open(&self) -> bool {
        self.window.is_open()
    }

    #[inline]
    fn get_mouse_position(&self) -> Option<(f32, f32)> {
        self.window.get_mouse_pos(minifb::MouseMode::Pass)
    }
    #[inline]
    fn get_size(&self) -> (usize, usize) {
        self.window.get_size()
    }
    #[inline]
    fn is_key_down(&self, key: KeyCode) -> bool {
        self.window.is_key_down(map_key(key))
    }
    #[inline]
    fn is_mouse_down(&self, button: MouseButton) -> bool {
        self.window.get_mouse_down(map_mouse(button))
    }
    fn get_mouse_scroll(&self) -> Option<(f32, f32)> {
        self.window.get_scroll_wheel()
    }
    #[inline]
    fn set_title(&mut self, title: &str) {
        self.window.set_title(title);
    }
    #[inline]
    fn get_time(&self) -> Box<(dyn Time + 'static)> {
        // Native environment: using SystemTime to get current time
        let now = std::time::Instant::now();
        return Box::new(NativeTime {
            time: now,
        });
    }
    #[inline]
    fn wait(&self, time: u64) {
        std::thread::sleep(Duration::from_millis(time));
    }
    #[inline]
    fn set_target_fps(&mut self, fps: usize) {
        self.window.set_target_fps(fps);
    }
    #[inline]
    fn set_always_ontop(&mut self, always_ontop: bool) {
        self.window.topmost(always_ontop);
    }
    #[inline]
    fn set_position(&mut self, x: isize, y: isize) {
        self.window.set_position(x, y);
    }
    #[inline]
    fn get_position(&self) -> (isize, isize) {
        self.window.get_position()
    }
    #[inline]
    fn set_icon(&mut self, buffer: &[u32], width: u32, height: u32) {
        // assert_eq!(
        //     buffer.len(),
        //     (width * height) as usize,
        //     "Buffer size doesn't match dimensions"
        // );
        // let buffer64: Vec<u64> = buffer.iter().map(|&x| x as u64).collect();

        // let boxed_buffer = buffer64.into_boxed_slice();

        // // Leak the memory (intentionally) so it persists for the lifetime of the application or whatever the hell that means
        // let leaked_buffer = Box::leak(boxed_buffer);

        // let icon = minifb::Icon::Buffer(
        //     leaked_buffer.as_ptr(),
        //     leaked_buffer.len() as u32,
        // );
        // self.window.set_icon(icon);
        #[cfg(target_os = "windows")]
        {
            let ico_data = encode_to_ico_format(buffer, width, height);

            let temp_dir = std::env::temp_dir();
            let ico_path = temp_dir.join("temp_icon.ico");

            std::fs::write(&ico_path, &ico_data)
                .expect("Failed to write temporary icon file");

            self.window.set_icon(
                minifb::Icon::from_str(ico_path.to_str().unwrap()).unwrap(),
            );
        }

        // For non-Windows platforms, try the buffer approach
        #[cfg(not(target_os = "windows"))]
        {
            let buffer64: Vec<u64> = buffer.iter().map(|&x| x as u64).collect();
            let boxed_buffer = buffer64.into_boxed_slice();
            let leaked_buffer = Box::leak(boxed_buffer);

            // Different platforms might expect different Icon constructor signatures
            // You might need to check minifb source to see exact parameters needed
            let icon = minifb::Icon::Buffer(
                leaked_buffer.as_ptr(),
                leaked_buffer.len() as u32,
                width,
                height,
            );

            self.window.set_icon(icon);
        }
    }
}

pub struct NativeTime {
    time: std::time::Instant,
}
impl Time for NativeTime {
    fn get_elapsed_time(&self) -> u64 {
        self.time.elapsed().as_millis() as u64
    }
}
pub const fn from_micros_u128(micros: u128) -> std::time::Duration {
    // We can safely break the u128 into two u64s.
    let secs = (micros / 1_000_000) as u64; // seconds
    let nanos = ((micros % 1_000_000) * 1000) as u32; // nanoseconds (from microseconds)

    std::time::Duration::new(secs, nanos)
}
fn encode_to_ico_format(buffer: &[u32], width: u32, height: u32) -> Vec<u8> {
    use ico::{IconDir, IconDirEntry, IconImage, ResourceType};

    // Create a new icon directory
    let mut icon_dir = IconDir::new(ResourceType::Icon);

    // Convert the RGBA u32 buffer to a Vec<u8> in BGRA format
    // Windows .ico format typically expects BGRA ordering
    let mut image_data = Vec::with_capacity(buffer.len() * 4);

    for &pixel in buffer {
        // Extract RGBA components from u32
        let r = ((pixel >> 16) & 0xFF) as u8;
        let g = ((pixel >> 8) & 0xFF) as u8;
        let b = (pixel & 0xFF) as u8;
        let _a = ((pixel >> 24) & 0xFF) as u8; // ALPHA IS NOT READ CORRECTLY -> IT'S ALWAYS 0
        println!("Fix alpha channel not being read correctly");

        // Push as BGRA
        image_data.push(b);
        image_data.push(g);
        image_data.push(r);
        image_data.push(255);
    }

    // Create icon image with proper transparency
    let icon_image = IconImage::from_rgba_data(width, height, image_data);

    // Add the image to the icon directory
    icon_dir.add_entry(
        IconDirEntry::encode(&icon_image).expect("Failed to encode icon image"),
    );

    // Encode the icon directory to a Vec<u8>
    let mut ico_data = Vec::new();
    icon_dir.write(&mut ico_data).expect("Failed to write icon data");

    ico_data
}
pub struct NativeFileSystem {}
impl NativeFileSystem {
    pub fn new() -> Self {
        Self {}
    }
}
use std::io::Read;

impl FileSystem for NativeFileSystem {
    fn get_file_contents(
        &self,
        path: &str,
    ) -> Result<FileData, Box<dyn std::error::Error>> {
        let mut file = std::fs::File::open(path)?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        Ok(FileData::from_bytes(buffer))
    }

    fn write_to_file(&self, path: &str, contents: &str) {
        std::fs::write(path, contents).expect("Failed to write file");
    }
    fn get_files_in_folder(&self, path: &str) -> Vec<String> {
        let mut files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.filter_map(Result::ok) {
                let entry_path = entry.path();
                if entry_path.is_file() {
                    if let Some(file_name) = entry_path.file_name() {
                        files.push(file_name.to_string_lossy().to_string());
                    }
                }
            }
        }
        return files;
    }

    fn get_folders_in_folder(&self, path: &str) -> Vec<String> {
        let mut folders = Vec::new();
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.filter_map(Result::ok) {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    if let Some(folder_name) = entry_path.file_name() {
                        folders.push(folder_name.to_string_lossy().to_string());
                    }
                }
            }
        }
        return folders;
    }
    fn join(&self, path1: &str, path2: &str) -> String {
        return format!("{}/{}", path1, path2);
    }
    fn does_file_exist(&self, path: &str) -> bool {
        std::fs::metadata(path).is_ok()
    }
}

// Compile-time key mapping function
const fn map_mouse(button: MouseButton) -> minifb::MouseButton {
    match button {
        MouseButton::Left => minifb::MouseButton::Left,
        MouseButton::Right => minifb::MouseButton::Right,
        MouseButton::Middle => minifb::MouseButton::Middle,
    }
}
const fn map_key(key: KeyCode) -> minifb::Key {
    match key {
        // Letters
        KeyCode::A => minifb::Key::A,
        KeyCode::B => minifb::Key::B,
        KeyCode::C => minifb::Key::C,
        KeyCode::D => minifb::Key::D,
        KeyCode::E => minifb::Key::E,
        KeyCode::F => minifb::Key::F,
        KeyCode::G => minifb::Key::G,
        KeyCode::H => minifb::Key::H,
        KeyCode::I => minifb::Key::I,
        KeyCode::J => minifb::Key::J,
        KeyCode::K => minifb::Key::K,
        KeyCode::L => minifb::Key::L,
        KeyCode::M => minifb::Key::M,
        KeyCode::N => minifb::Key::N,
        KeyCode::O => minifb::Key::O,
        KeyCode::P => minifb::Key::P,
        KeyCode::Q => minifb::Key::Q,
        KeyCode::R => minifb::Key::R,
        KeyCode::S => minifb::Key::S,
        KeyCode::T => minifb::Key::T,
        KeyCode::U => minifb::Key::U,
        KeyCode::V => minifb::Key::V,
        KeyCode::W => minifb::Key::W,
        KeyCode::X => minifb::Key::X,
        KeyCode::Y => minifb::Key::Y,
        KeyCode::Z => minifb::Key::Z,

        // Numbers
        KeyCode::Num0 => minifb::Key::Key0,
        KeyCode::Num1 => minifb::Key::Key1,
        KeyCode::Num2 => minifb::Key::Key2,
        KeyCode::Num3 => minifb::Key::Key3,
        KeyCode::Num4 => minifb::Key::Key4,
        KeyCode::Num5 => minifb::Key::Key5,
        KeyCode::Num6 => minifb::Key::Key6,
        KeyCode::Num7 => minifb::Key::Key7,
        KeyCode::Num8 => minifb::Key::Key8,
        KeyCode::Num9 => minifb::Key::Key9,
        KeyCode::KeyPad0 => minifb::Key::NumPad0,
        KeyCode::KeyPad1 => minifb::Key::NumPad1,
        KeyCode::KeyPad2 => minifb::Key::NumPad2,
        KeyCode::KeyPad3 => minifb::Key::NumPad3,
        KeyCode::KeyPad4 => minifb::Key::NumPad4,
        KeyCode::KeyPad5 => minifb::Key::NumPad5,
        KeyCode::KeyPad6 => minifb::Key::NumPad6,
        KeyCode::KeyPad7 => minifb::Key::NumPad7,
        KeyCode::KeyPad8 => minifb::Key::NumPad8,
        KeyCode::KeyPad9 => minifb::Key::NumPad9,

        // Function Keys
        KeyCode::F1 => minifb::Key::F1,
        KeyCode::F2 => minifb::Key::F2,
        KeyCode::F3 => minifb::Key::F3,
        KeyCode::F4 => minifb::Key::F4,
        KeyCode::F5 => minifb::Key::F5,
        KeyCode::F6 => minifb::Key::F6,
        KeyCode::F7 => minifb::Key::F7,
        KeyCode::F8 => minifb::Key::F8,
        KeyCode::F9 => minifb::Key::F9,
        KeyCode::F10 => minifb::Key::F10,
        KeyCode::F11 => minifb::Key::F11,
        KeyCode::F12 => minifb::Key::F12,
        KeyCode::F13 => minifb::Key::F13,
        KeyCode::F14 => minifb::Key::F14,
        KeyCode::F15 => minifb::Key::F15,

        KeyCode::F16 => minifb::Key::Unknown,
        KeyCode::F17 => minifb::Key::Unknown,
        KeyCode::F18 => minifb::Key::Unknown,
        KeyCode::F19 => minifb::Key::Unknown,
        KeyCode::F20 => minifb::Key::Unknown,
        KeyCode::F21 => minifb::Key::Unknown,
        KeyCode::F22 => minifb::Key::Unknown,
        KeyCode::F23 => minifb::Key::Unknown,
        KeyCode::F24 => minifb::Key::Unknown,

        // Modifiers
        KeyCode::LeftShift => minifb::Key::LeftShift,
        KeyCode::RightShift => minifb::Key::RightShift,
        KeyCode::LeftControl => minifb::Key::LeftCtrl,
        KeyCode::RightControl => minifb::Key::RightCtrl,
        KeyCode::LeftAlt => minifb::Key::LeftAlt,
        KeyCode::RightAlt => minifb::Key::RightAlt,
        KeyCode::LeftSuper => minifb::Key::LeftSuper,
        KeyCode::RightSuper => minifb::Key::RightSuper,

        // Symbols
        KeyCode::Space => minifb::Key::Space,
        KeyCode::Enter => minifb::Key::Enter,
        KeyCode::Escape => minifb::Key::Escape,
        KeyCode::Backspace => minifb::Key::Backspace,
        KeyCode::Tab => minifb::Key::Tab,

        // Arrows
        KeyCode::Up => minifb::Key::Up,
        KeyCode::Down => minifb::Key::Down,
        KeyCode::Left => minifb::Key::Left,
        KeyCode::Right => minifb::Key::Right,

        // Extra
        KeyCode::Comma => minifb::Key::Comma,
        KeyCode::Period => minifb::Key::Period,
        KeyCode::Minus => minifb::Key::Minus,
        KeyCode::Equals => minifb::Key::Equal,
        KeyCode::LeftBracket => minifb::Key::LeftBracket,
        KeyCode::RightBracket => minifb::Key::RightBracket,
        KeyCode::Backslash => minifb::Key::Backslash,
        KeyCode::Semicolon => minifb::Key::Semicolon,
        KeyCode::Quote => minifb::Key::Apostrophe,
        KeyCode::Tilde => minifb::Key::Unknown,

        // Other letters
        KeyCode::Ä => minifb::Key::Unknown,
        KeyCode::Ü => minifb::Key::Unknown,
        KeyCode::Ö => minifb::Key::Unknown,
        KeyCode::SS => minifb::Key::Unknown,
        KeyCode::Â => minifb::Key::Unknown,
        KeyCode::Ú => minifb::Key::Unknown,
        KeyCode::Ô => minifb::Key::Unknown,
        KeyCode::Î => minifb::Key::Unknown,
        KeyCode::Ê => minifb::Key::Unknown,
        KeyCode::Ð => minifb::Key::Unknown,
        KeyCode::Œ => minifb::Key::Unknown,
        KeyCode::Á => minifb::Key::Unknown,
        KeyCode::Ý => minifb::Key::Unknown,
        KeyCode::Ï => minifb::Key::Unknown,
        KeyCode::Ñ => minifb::Key::Unknown,
        KeyCode::Ò => minifb::Key::Unknown,

        // Other
        KeyCode::ScrollLock => minifb::Key::ScrollLock,
        // Lock keys (new)
        KeyCode::CapsLock => minifb::Key::CapsLock,
        KeyCode::NumLock => minifb::Key::NumLock,

        // Editing keys
        KeyCode::Insert => minifb::Key::Insert,
        KeyCode::Delete => minifb::Key::Delete,
        KeyCode::Home => minifb::Key::Home,
        KeyCode::End => minifb::Key::End,
        KeyCode::PageUp => minifb::Key::PageUp,
        KeyCode::PageDown => minifb::Key::PageDown,

        // Keypad ops
        KeyCode::KeyPadDivide => minifb::Key::Slash,
        KeyCode::KeyPadMultiply => minifb::Key::Unknown,
        KeyCode::KeyPadSubtract => minifb::Key::Minus,
        KeyCode::KeyPadAdd => minifb::Key::Unknown,
        KeyCode::KeyPadDecimal => minifb::Key::Unknown,
        KeyCode::KeyPadEnter => minifb::Key::Enter,

        // Multimedia keys (unsupported in minifb)
        KeyCode::MediaPlayPause => minifb::Key::Unknown,
        KeyCode::MediaStop => minifb::Key::Unknown,
        KeyCode::MediaNext => minifb::Key::Unknown,
        KeyCode::MediaPrev => minifb::Key::Unknown,
        KeyCode::VolumeUp => minifb::Key::Unknown,
        KeyCode::VolumeDown => minifb::Key::Unknown,
        KeyCode::Mute => minifb::Key::Unknown,

        // Browser/OS keys
        KeyCode::BrowserBack => minifb::Key::Unknown,
        KeyCode::BrowserForward => minifb::Key::Unknown,
        KeyCode::BrowserRefresh => minifb::Key::Unknown,
        KeyCode::BrowserHome => minifb::Key::Unknown,
        KeyCode::LaunchMail => minifb::Key::Unknown,
        KeyCode::LaunchApp1 => minifb::Key::Unknown,
        KeyCode::LaunchApp2 => minifb::Key::Unknown,

        // Platform-specific
        KeyCode::Menu => minifb::Key::Menu,
        KeyCode::PrintScreen => minifb::Key::Unknown,
        KeyCode::Pause => minifb::Key::Pause,
        KeyCode::Application => minifb::Key::Unknown,

        // Symbols not mapped before
        KeyCode::Slash => minifb::Key::Slash,
        KeyCode::Grave => minifb::Key::Unknown,
        // Fallback
        // _ => minifb::Key::Unknown,
    }
}

pub fn load_font(path: &str) -> fontdue::Font {
    let font_data = std::fs::read(path).expect("Failed to read font file");
    fontdue::Font::from_bytes(font_data, fontdue::FontSettings::default())
        .expect("Failed to parse font")
}
