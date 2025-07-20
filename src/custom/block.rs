use fontdue::Font;
use std::cell::Cell;
use std::cell::RefCell; // Cell but with & ?

use crate::custom::BlockInput;
use crate::custom::WorkSpace;
use crate::custom::ID;
use crate::logic::Physics;

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
    pub input_offsets: RefCell<Vec<f32>>,
    pub block_color_id: usize,
    pub id: ID,
    pub connected_top: Cell<Option<ID>>,
    pub connected_below: Cell<Option<ID>>,
    pub possible_connection_above: Cell<Option<ID>>,
    pub possible_connection_below: Cell<Option<ID>>,
    pub recently_moved: Cell<bool>,
}
impl Block {
    pub fn new<L: Physics>(
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
        output_color_names: &[String],
        font: &Font,
        workspace: &mut WorkSpace<L>,
    ) -> Block {
        let color_id = output_color_names
            .iter()
            .position(|x| *x == output)
            .expect("Could not find color name");
        let x = ((u16::MAX / 2) as i16 + x) as u16;
        let y = ((u16::MAX / 2) as i16 + y) as u16;

        if name.matches("{}").count() != inputs.len() {
            panic!(
                "Name expects {} inputs while an unfitting {} were provided for {} ('{}', '{:?}')",
                name.matches("{}").count(),
                inputs.len(),
                internal_name,
                name,inputs
            )
        }
        let text_between_inputs: Vec<String> = name
            .split("{}")
            .collect::<Vec<&str>>()
            .iter()
            .map(|x| x.to_string())
            .collect();

        let input_offsets = Vec::new();

        // for i in name.split("{}") {
        //     // Keep the i == "" because otherwise rendering is wonky ._.
        //     framework.log("{}", i);
        //     text_between_inputs.push(i.to_string());
        //     // input_offsets.push(
        //     //     get_length_of_text_in_font(i, font)
        //     //         + input_offsets.iter().sum::<f32>(),
        //     // );
        // }
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
            input_offsets: RefCell::new(input_offsets),
            block_color_id: color_id,
            id: workspace.increment_block_id().into(),
            connected_top: Cell::new(None),
            connected_below: Cell::new(None),
            possible_connection_above: Cell::new(None),
            possible_connection_below: Cell::new(None),
            recently_moved: Cell::new(false),
        };
        b.recalculate_input_offsets(font);
        b.recalculate_width(font);
        b
    }
    pub fn recalculate_width(&self, font: &Font) {
        let mut width = 0.0;
        let offset_length = self.input_offsets.borrow().len();
        if offset_length > 0 {
            width += self.input_offsets.borrow()[offset_length - 1];
        }
        // let inp_len = self.inputs.len();
        // if inp_len > 0 {
        //     width += self.inputs[inp_len - 1].get_width(font);
        // }
        width += mirl::render::get_length_of_string(
            &self.name[self.name.len() - 1],
            self.height.get() / 2.0,
            font,
        );
        // Get the last known input offset, add the width of the input, and add the letters after the input
        self.width.set(width);
    }
    pub fn recalculate_input_offsets(&self, font: &Font) {
        let mut offsets = Vec::new();
        let mut total_offset = 0.0;
        let loop_amount = self.name.len() - 1;
        offsets.push(total_offset);
        if loop_amount > 0 {
            for i in 0..loop_amount {
                // Get text
                let before_text = &self.name[i];
                // Get length of text
                total_offset += mirl::render::get_length_of_string(
                    &before_text,
                    self.height.get() / 2.0,
                    font,
                );
                // Add offset of text
                offsets.push(total_offset);
                // Get width of input
                let input = &self.inputs[i];
                // Add offset of input
                total_offset += input.get_width(font);
                // Add offset of input
                offsets.push(total_offset);
            }
            // let before_text = &self.name[loop_amount];
            // // Get length of text
            // total_offset += mirl::render::get_length_of_string(
            //     &before_text,
            //     self.height.get() / 2.0,
            //     font,
            // );
            // // Add offset of text
            // offsets.push(total_offset);
            if offsets.len() != self.name.len() + self.inputs.len() {
                panic!(
                    "offset {:#?} | {:#?}, {}/{}",
                    offsets,
                    self.inputs,
                    offsets.len(),
                    loop_amount * 2
                );
            }
            // framework.log("{:?}", offsets);
            // framework.log("{:?}", self.name);
            // framework.log("{:?}", self.inputs.len());
            // std::process::exit(0);
            self.input_offsets.swap(&RefCell::new(offsets));
        }
    }
}
