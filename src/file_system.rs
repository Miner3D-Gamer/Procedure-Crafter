use fontdue::{Font, FontSettings};

pub fn load_font(path: &str) -> Font {
    let font_data = std::fs::read(path).expect("Failed to read font file");
    Font::from_bytes(font_data, FontSettings::default())
        .expect("Failed to parse font")
}
pub fn get_file_contents(path: &str) -> String {
    std::fs::read_to_string(path).expect("Failed to read file")
}
pub fn write_to_file(path: &str, contents: &str) {
    std::fs::write(path, contents).expect("Failed to write file");
}
