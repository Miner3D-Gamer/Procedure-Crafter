use fontdue::Font;
use std::cell::Cell;

use crate::all::get_length_of_text_in_font;
use crate::custom::BlockInput;
use crate::custom::WorkSpace;
use crate::custom::ID;
use crate::logic::Physics;
use crate::render::RenderSettings;

#[derive(Clone, Debug)]
pub struct Block {
    pub name: Vec<String>,
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
    pub output: String,
    pub inputs: Vec<BlockInput>,
    pub input_offsets: Vec<f32>,
    pub block_color_id: usize,
    pub id: ID,
    pub connected_top: Cell<Option<ID>>,
    pub connected_below: Cell<Option<ID>>,
    pub possible_connection_above: Cell<Option<ID>>,
    pub possible_connection_below: Cell<Option<ID>>,
    pub recently_moved: Cell<bool>,
}
impl Block {
    pub fn new<S: RenderSettings, L: Physics>(
        name: String,
        internal_name: String,
        x: i16,
        y: i16,
        block_type: u8,
        required_imports: Vec<String>,
        required_contexts: Vec<String>,
        file_versions: Vec<String>,
        output: String,
        inputs: Vec<BlockInput>,
        outuput_color_names: &Vec<String>,
        font: &Font,
        workspace: &mut WorkSpace<S, L>,
    ) -> Block {
        let color_id = outuput_color_names
            .iter()
            .position(|x| *x == output)
            .expect("Could not find color name");
        let x = ((u16::MAX / 2) as i16 + x) as u16;
        let y = ((u16::MAX / 2) as i16 + y) as u16;

        let mut text_between_inputs = Vec::new();
        let mut input_offsets = Vec::new();

        if name.matches("{}").count() != inputs.len() {
            panic!(
                "Name expects {} inputs while an unfitting {} were provided for {} ('{}', '{:?}')",
                name.matches("{}").count(),
                inputs.len(),
                internal_name,
                name,inputs
            )
        }

        for i in name.split("{}") {
            // Keep the i == "" because otherwise rendering is wonky ._.
            text_between_inputs.push(i.to_string());
            input_offsets.push(get_length_of_text_in_font(i, font));
        }

        let b = Block {
            name: text_between_inputs,
            internal_name: internal_name,
            x: Cell::new(x),
            y: Cell::new(y),
            width: Cell::new(0.0),
            height: Cell::new(40.0),
            block_type: block_type,
            required_imports: required_imports,
            required_contexts: required_contexts,
            file_versions: file_versions,
            output: output,
            inputs: inputs,
            input_offsets: input_offsets,
            block_color_id: color_id,
            id: workspace.increment_block_id().into(),
            connected_top: Cell::new(None),
            connected_below: Cell::new(None),
            possible_connection_above: Cell::new(None),
            possible_connection_below: Cell::new(None),
            recently_moved: Cell::new(false),
        };
        b.recalculate_width(font);
        return b;
    }
    pub fn recalculate_width(&self, font: &Font) {
        let mut temp = 0.0;
        for i in &self.input_offsets {
            temp += i
        }
        for i in &self.inputs {
            temp += i.get_width(font)
        }

        self.width.set(temp);
    }
}
