use fontdue::Font;
use std::cell::Cell;

use crate::custom::BlockInput;
use crate::custom::ID;
use crate::idk::WorkSpace;
use crate::render::RenderSettings;

pub struct Block {
    pub name: String,
    pub internal_name: String,
    pub x: Cell<u16>,
    pub y: Cell<u16>,
    pub width: Cell<f32>,
    pub height: Cell<f32>,
    pub block_type: u8,
    // 0: Action, 1: Inline, 2: Hugging, 3: Event
    pub required_imports: Vec<String>,
    pub required_contexts: Vec<String>,
    pub file_versions: Vec<String>,
    pub file_locations: Vec<String>,
    pub output: String,
    pub inputs: Vec<BlockInput>,
    pub block_color_id: usize,
    pub id: ID,
    pub connected_top: Cell<Option<ID>>,
    pub connected_below: Cell<Option<ID>>,
    pub possible_connection_above: Cell<Option<ID>>,
    pub possible_connection_below: Cell<Option<ID>>,
    pub recently_moved: Cell<bool>,
}
impl Block {
    pub fn new<S: RenderSettings>(
        name: String,
        internal_name: String,
        x: i16,
        y: i16,
        block_type: u8,
        required_imports: Vec<String>,
        required_contexts: Vec<String>,
        file_versions: Vec<String>,
        file_locations: Vec<String>,
        output: String,
        inputs: Vec<BlockInput>,
        outuput_color_names: &Vec<String>,
        font: &Font,
        workspace: &mut WorkSpace<S>,
    ) -> Block {
        let color_id = outuput_color_names
            .iter()
            .position(|x| *x == output)
            .expect("Could not find color name");
        let x = ((u16::MAX / 2) as i16 + x) as u16;
        let y = ((u16::MAX / 2) as i16 + y) as u16;

        Block {
            name: name.clone(),
            internal_name: internal_name,
            x: Cell::new(x),
            y: Cell::new(y),
            width: Cell::new(get_length_of_text_in_font(&name, &font)),
            height: Cell::new(40.0),
            block_type: block_type,
            required_imports: required_imports,
            required_contexts: required_contexts,
            file_versions: file_versions,
            file_locations: file_locations,
            output: output,
            inputs: inputs,
            block_color_id: color_id,
            id: workspace.increment_block_id().into(),
            connected_top: Cell::new(None),
            connected_below: Cell::new(None),
            possible_connection_above: Cell::new(None),
            possible_connection_below: Cell::new(None),
            recently_moved: Cell::new(false),
        }
    }
}

fn get_length_of_text_in_font(text: &str, font: &Font) -> f32 {
    let mut length = 0.0;
    for ch in text.chars() {
        let (metrics, _) = font.rasterize(ch, 20.0);
        length += metrics.advance_width;
    }
    return length;
}
