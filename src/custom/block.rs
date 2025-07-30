use fontdue::Font;
use mirl::extensions::TupleOps;
use std::cell::Cell;
use std::cell::RefCell; // Cell but with & ?

use crate::all::get_top_most_block_id_or_self;
use crate::all::index_by_block_id;
use crate::custom::BlockInput;
use crate::custom::WorkSpace;
use crate::custom::ID;
use crate::logic::Physics;

use derive_more::Debug;

#[derive(Clone, PartialEq, Debug)]
pub struct Block {
    #[debug(skip)]
    pub name: Vec<String>,
    pub internal_name: String,
    #[debug("{:?}",x.get())]
    pub x: Cell<u16>,
    #[debug("{:?}",y.get())]
    pub y: Cell<u16>,
    #[debug("{:?}",width.get())]
    pub width: Cell<f32>,
    #[debug("{:?}",height.get())]
    pub height: Cell<f32>,
    pub block_type: u8,
    // 0: Action, 1: Inline, 2: Event
    pub required_imports: Vec<String>,
    pub required_contexts: Vec<String>,
    pub file_versions: Vec<String>,
    pub output: String,
    pub inputs: Vec<BlockInput>,
    #[debug(skip)]
    pub input_offsets: RefCell<Vec<f32>>,
    pub stored_inputs: RefCell<Vec<Option<ID>>>,
    #[debug(skip)]
    pub block_color_id: usize,
    #[debug("{id:?}")]
    pub id: ID,
    #[debug("{:?}",connected_top.get())]
    pub connected_top: Cell<Option<ID>>,
    #[debug("{:?}",connected_above.get())]
    pub connected_above: Cell<Option<ID>>,
    #[debug("{:?}",connected_below.get())]
    pub connected_below: Cell<Option<ID>>,
    #[debug(skip)]
    pub possible_connection_above: Cell<Option<ID>>,
    #[debug(skip)]
    pub possible_connection_below: Cell<Option<ID>>,
    #[debug("{:?}", recently_moved.get())]
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
        let mut stored = Vec::new();
        for _ in 0..inputs.len() {
            stored.push(None)
        }

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
            internal_name,
            x: Cell::new(x),
            y: Cell::new(y),
            width: Cell::new(0.0),
            height: Cell::new(40.0),
            block_type,
            required_imports,
            required_contexts,
            file_versions,
            output,
            inputs,
            stored_inputs: RefCell::new(stored),
            input_offsets: RefCell::new(input_offsets),
            block_color_id: color_id,
            id: workspace.increment_block_id().into(),
            connected_top: Cell::new(None),
            connected_above: Cell::new(None),
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
        width += mirl::render::get_length_of_string(
            &self.name[self.name.len() - 1],
            self.height.get() / 2.0,
            font,
        );
        // Get the last known input offset, add the width of the input, and add the letters after the input
        self.width.set(width);
    }
    pub fn disconnect_below(&self, blocks: &Vec<Block>) {
        if let Some(below) = self.connected_below.get() {
            let index_of_connected_below =
                index_by_block_id(&below, blocks).unwrap();
            let block_below = &blocks[index_of_connected_below];
            block_below.connected_top.set(None);
            block_below.connected_above.set(None);
            block_below.update_topmost(blocks, true);
        }
    }
    pub fn disconnect_above(&self, blocks: &Vec<Block>) {
        if let Some(above) = self.connected_above.get() {
            let block_above =
                &blocks[index_by_block_id(&above, blocks).unwrap()];
            block_above.connected_below.set(None);
            self.connected_above.set(None);
        }
        self.connected_top.set(None);
    }
    pub fn connect_below_to_above(&self, blocks: &Vec<Block>) {
        if self.connected_above.get().is_none() {
            self.disconnect_below(blocks);
            return;
        }
        if self.connected_below.get().is_none() {
            self.disconnect_above(blocks);
            return;
        }
        let block_above = &blocks[index_by_block_id(
            &self.connected_above.get().unwrap(),
            blocks,
        )
        .unwrap()];
        let block_below = &blocks[index_by_block_id(
            &self.connected_below.get().unwrap(),
            blocks,
        )
        .unwrap()];

        self.connected_top.set(None);
        self.connected_above.set(None);
        self.connected_below.set(None);
        block_above.connected_below.set(Some(block_below.id));
        block_below.connected_above.set(Some(block_above.id));
        block_below.update_topmost(blocks, true);
    }
    pub fn update_topmost(&self, blocks: &Vec<Block>, set_moved: bool) {
        let new_id = get_top_most_block_id_or_self(
            blocks,
            index_by_block_id(&self.id, blocks).unwrap(),
        );
        self.recursive_set_topmost(blocks, new_id, set_moved);
    }
    pub fn recursive_set_topmost(
        &self,
        blocks: &Vec<Block>,
        id: ID,
        set_moved: bool,
    ) {
        if set_moved {
            self.recently_moved.set(true)
        }
        if self.id != id {
            self.connected_top.set(Some(id));
        }
        if let Some(below) = self.connected_below.get() {
            blocks[index_by_block_id(&below, blocks).unwrap()]
                .recursive_set_topmost(blocks, id, set_moved);
        }
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
                    before_text,
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
            // // Add offset of text
            if offsets.len() != self.name.len() + self.inputs.len() {
                panic!(
                    "offset {:#?} | {:#?}, {}/{}",
                    offsets,
                    self.inputs,
                    offsets.len(),
                    loop_amount * 2
                );
            }
            self.input_offsets.swap(&RefCell::new(offsets));
        }
    }
    pub fn get_all_ids_of_all_inputs(&self, blocks: &Vec<Block>) -> Vec<ID> {
        let mut found = Vec::new();
        for some_id in self.stored_inputs.borrow().iter().flatten() {
            found.push(*some_id);
            let index = index_by_block_id(some_id, blocks).unwrap();
            let input = &blocks[index];
            found.extend(input.get_all_ids_of_all_inputs(blocks));
        }

        found
    }
    pub fn get_inputs_in_range<L: Physics>(
        &self,
        point: (f32, f32),
        offset: (f32, f32),
        range: f32,
        logic: &L,
        backlist: &[ID],
        blocks: &Vec<Block>,
    ) -> Option<InputRememberer> {
        if backlist.contains(&self.id) {
            return None;
        }
        let mut found = Vec::new();
        for i in 0..self.inputs.len() {
            let item = self.input_offsets.borrow()[i * 2];
            let point_x = self.x.get() as f32 + item;
            // This checks the corner of the boxes, not the middle
            let distance = logic.get_distance_between_positions(
                point.0 + offset.0,
                point.1 + offset.1,
                point_x,
                self.y.get() as f32,
            );
            if distance > range {
                continue;
            }
            if let Some(stored_input) = self.stored_inputs.borrow()[i] {
                found.push((
                    i,
                    blocks[index_by_block_id(&stored_input, blocks).unwrap()]
                        .get_inputs_in_range(
                            point,
                            offset.add((item, 0.0)),
                            range,
                            logic,
                            backlist,
                            blocks,
                        ),
                    Some(distance),
                ))
            } else {
                found.push((i, None, Some(distance)))
            }
        }
        let ir = InputRememberer {
            internal_inputs: found,
            own_id: self.id,
        };
        Some(ir)
    }
    pub fn create_input_rememberer(
        &self,
        blocks: &Vec<Block>,
    ) -> InputRememberer {
        let mut found = Vec::new();
        for (idx, id) in self.stored_inputs.borrow().iter().enumerate() {
            if let Some(id) = id {
                found.push((
                    idx,
                    Some(
                        blocks[index_by_block_id(id, blocks).unwrap()]
                            .create_input_rememberer(blocks),
                    ),
                    None,
                ))
            }
        }

        InputRememberer {
            own_id: self.id,
            internal_inputs: found,
        }
    }
}
pub struct InputRememberer {
    pub internal_inputs: Vec<(usize, Option<InputRememberer>, Option<f32>)>,
    pub own_id: ID,
}
