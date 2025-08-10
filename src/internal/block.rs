use fontdue::Font;
use mirl::extensions::TupleOps;
use num_traits::ToPrimitive;
use std::cell::Cell;
use std::cell::RefCell;

use crate::all::get_bottom_most_block_idx_or_self;
use crate::all::get_top_most_block_idx_or_self;
use crate::all::index_by_block_id;
use crate::internal::id::UsizeGetID;
use crate::internal::BlockInput;
use crate::internal::WorkSpace;
use crate::internal::ID;
use crate::logic::Physics;
use crate::CoordinateType;
use crate::CoordinateTypeSigned;
use crate::SizeType;
use mirl::extensions::*;

use derive_more::Debug;

#[derive(Clone, PartialEq, Debug)]
pub struct Block {
    #[debug(skip)]
    /// Name segments -> Split by input
    pub name: Vec<String>,
    /// Original name of the block
    pub original_name: String,
    /// Internal name for file accessing
    pub internal_name: String,
    #[debug("{:?}",x.get())]
    /// X coordinate
    pub x: Cell<CoordinateType>,
    #[debug("{:?}",y.get())]
    /// Y coordinate
    pub y: Cell<CoordinateType>,
    #[debug("{:?}",width.get())]
    /// Width
    pub width: Cell<SizeType>,
    #[debug("{:?}",height.get())]
    /// Height
    pub height: Cell<SizeType>,
    /// 0: Action (Main structure blocks)
    ///
    /// 1: Inline (Inputs)
    ///
    /// 2: Event (Entry points)
    pub block_type: u8,
    pub required_imports: Vec<String>,
    pub required_contexts: Vec<String>,
    pub file_versions: Vec<String>,
    pub output: String,
    pub inputs: Vec<BlockInput>,
    #[debug(skip)]
    pub input_offsets: RefCell<Vec<SizeType>>,
    #[debug("{:?}",stored_inputs.borrow())]
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
use std::ops::Div;
impl Block {
    pub fn new<L: Physics>(
        name: String,
        internal_name: String,
        x: CoordinateTypeSigned,
        y: CoordinateTypeSigned,
        block_type: u8,
        required_imports: Vec<String>,
        required_contexts: Vec<String>,
        file_versions: Vec<String>,
        output: String,
        inputs: Vec<BlockInput>,
        output_color_names: &[String],
        font: &Font,
        workspace: &mut WorkSpace<L>,
        overwrite_id: Option<ID>,
    ) -> Block {
        let color_id = output_color_names
            .iter()
            .position(|x| *x == output)
            .expect("Could not find color name");

        let x = x.map_sign_to_non_sign();
        let y = y.map_sign_to_non_sign();

        if name.matches("{}").count() != inputs.len() {
            panic!(
                "Translation name expects {} inputs while an unfitting {} were provided to the functions for '{}' \n(Raw name: '{}'\nRaw given inputs: '{:#?}')",
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
            original_name: name,
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
            id: if let Some(overwrite_id) = overwrite_id {
                overwrite_id
            } else {
                workspace.increment_block_id().into()
            },
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
        let mut width: SizeType = 0.0;
        let offset_length = self.input_offsets.borrow().len();
        if offset_length > 0 {
            width += self.input_offsets.borrow()[offset_length - 1];
        }
        width += mirl::render::get_length_of_string(
            &self.name[self.name.len() - 1],
            self.height.get().div(2.0).to_f32().unwrap(),
            font,
        ) as SizeType;
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
    /// Sets [`connected_below`](Block::connected_below) of [`connected_above`](Block::connected_above) to None
    ///
    /// Updates the [`connected_top`](Block::connected_top) of all connected blocks
    pub fn disconnect_above(&self, blocks: &Vec<Block>) {
        if let Some(above) = self.connected_above.get() {
            if let Some(idx) = index_by_block_id(&above, blocks) {
                let block_above = &blocks[idx];
                block_above.connected_below.set(None);
                self.connected_above.set(None);
            }
        }

        self.recursive_set_topmost(blocks, self.id, true);
    }
    pub fn connect_below_to_above(&self, blocks: &Vec<Block>) -> bool {
        if self.connected_above.get().is_none() {
            self.disconnect_below(blocks);
            return false;
        }
        if self.connected_below.get().is_none() {
            self.disconnect_above(blocks);
            return false;
        }
        if let Some(block_idx_above) =
            index_by_block_id(&self.connected_above.get().unwrap(), blocks)
        {
            if let Some(block_idx_below) =
                index_by_block_id(&self.connected_below.get().unwrap(), blocks)
            {
                let block_above = &blocks[block_idx_above];
                let block_below = &blocks[block_idx_below];

                // Disconnect self from all others
                self.connected_top.set(None);
                self.connected_above.set(None);
                self.connected_below.set(None);

                // Update block above
                block_above.connected_below.set(Some(block_below.id));

                // Update block below
                block_below.connected_above.set(Some(block_above.id));
                block_below.update_topmost(blocks, true);
                return true;
            }
        }
        false
    }
    pub fn update_topmost(&self, blocks: &Vec<Block>, set_moved: bool) {
        let new_id = get_top_most_block_idx_or_self(
            blocks,
            index_by_block_id(&self.id, blocks).unwrap(),
        );
        self.recursive_set_topmost(
            blocks,
            new_id.get_id_of_idx(blocks),
            set_moved,
        );
    }
    pub fn recursive_set_topmost(
        &self,
        blocks: &Vec<Block>,
        new_id: ID,
        set_moved: bool,
    ) -> bool {
        if set_moved {
            self.recently_moved.set(true)
        }
        if self.id != new_id {
            self.connected_top.set(Some(new_id));
        } else {
            self.connected_top.set(None);
        }
        if let Some(below) = self.connected_below.get() {
            if let Some(below_idx) = index_by_block_id(&below, blocks) {
                blocks[below_idx]
                    .recursive_set_topmost(blocks, new_id, set_moved);
                return true;
            }
        }
        false
    }
    pub fn connect_to_block(&self, block_idx: usize, blocks: &Vec<Block>) {
        let block_above = &blocks[block_idx];
        let block_below_id = block_above.connected_below.get();
        block_above.connected_below.set(Some(self.id));
        self.connected_above.set(Some(block_above.id));
        self.connected_top.set(block_above.connected_top.get());

        if let Some(block_below_id) = block_below_id {
            if let Some(block_below_idx) =
                index_by_block_id(&block_below_id, blocks)
            {
                let bottom_most = &blocks[get_bottom_most_block_idx_or_self(
                    blocks,
                    index_by_block_id(&self.id, blocks).unwrap(),
                )];
                bottom_most.connected_below.set(Some(block_below_id));
                let block_below = &blocks[block_below_idx];
                block_below.connected_above.set(Some(self.id))
            }
        }
    }
    pub fn connect_to_possibly_above(&self, blocks: &Vec<Block>) {
        if let Some(connection_id) = self.possible_connection_above.get() {
            if let Some(idx) = index_by_block_id(&connection_id, blocks) {
                self.connect_to_block(idx, blocks);
            }
        }
    }
    pub fn recalculate_input_offsets(&self, font: &Font) {
        let mut offsets: Vec<SizeType> = Vec::new();
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
                    self.height.get().div(2.0).to_f32().unwrap(),
                    font,
                ) as SizeType;
                // Add offset of text
                offsets.push(total_offset);
                // Get width of input
                let input = &self.inputs[i];
                // Add offset of input
                total_offset += input.get_width(font);
                // Add offset of input
                offsets.push(total_offset);
            }
            // Debug check if the right amount of stuff was added
            if offsets.len() != self.name.len() + self.inputs.len() {
                panic!(
                    "offset {:#?} | {:#?}, {}/{}",
                    offsets,
                    self.inputs,
                    offsets.len(),
                    loop_amount * 2
                );
            }
            // Add offset of text
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
        point: (SizeType, SizeType),
        offset: (SizeType, SizeType),
        range: SizeType,
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
            let point_x = self.x.get() as SizeType + item;
            // This checks the corner of the boxes, not the middle
            let distance = logic.get_distance_between_positions(
                point.0 + offset.0,
                point.1 + offset.1,
                point_x as SizeType,
                self.y.get() as SizeType,
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
    pub fn duplicate<L: Physics>(
        &self,
        output_color_names: &[String],
        font: &Font,
        workspace: &mut WorkSpace<L>,
    ) -> Self {
        Self::new(
            self.original_name.clone(),
            self.internal_name.clone(),
            self.x.get().map_non_sign_to_sign(),
            self.y.get().map_non_sign_to_sign(),
            self.block_type,
            self.required_imports.clone(),
            self.required_contexts.clone(),
            self.file_versions.clone(),
            self.output.clone(),
            self.inputs.clone(),
            output_color_names,
            font,
            workspace,
            None,
        )
    }
}
pub struct InputRememberer {
    pub internal_inputs:
        Vec<(usize, Option<InputRememberer>, Option<SizeType>)>,
    pub own_id: ID,
}
