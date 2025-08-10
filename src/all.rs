#![allow(dead_code)]
#![allow(static_mut_refs)] // Yeah, this is probably fine


// Bugs:
// The ghost render is messed up not allowing for valid connections

use core::panic;
use fontdue::Font;
use mirl::graphics::rgb_to_u32;
use mirl::lists::add_item_to_max_sized_list;
use mirl::platform::framework_traits::ExtendedFramework;
use mirl::platform::framework_traits::Framework;
// use mirl::platform::mouse::position::RawMouseInputTrait;
//use mirl::platform::framework_traits::Framework;
use mirl::platform::Buffer;
use mirl::platform::FileSystem;
use std::collections::HashMap;

use crate::internal::id::UsizeGetID;
use crate::internal::BlockInput;
use crate::internal::WorkSpace;
use crate::logic::Physics;

use crate::internal::Block;
use crate::internal::Camera;
use crate::internal::ID;
use crate::idk::draw_block;
use crate::CoordinateType;
use crate::SizeType;
use mirl::extensions::*;




type KeyMap = HashMap<&'static str, Vec<mirl::platform::KeyCode>>;
static KEY_BINDS: std::sync::LazyLock<std::sync::Mutex<KeyMap>> = std::sync::LazyLock::new(|| {
    std::sync::Mutex::new(KeyMap::new())
});


/// Adds or updates a key. Returns the previous value if any.
pub fn add_key_bind(
    key: &'static str,
    code: mirl::platform::KeyCode,
) -> Result<std::option::Option<std::vec::Vec<mirl::platform::KeyCode>>, String> {
    let mut keys = get_key_bind(key);
    let mut map = KEY_BINDS
        .lock()
        .map_err(|_| "Failed to lock KEY_BINDS (poisoned)".to_string())?;
    keys.push(code);

    Ok(map.insert(key, keys))
}

/// Removes a key. Returns the removed value if it existed.
pub fn remove_key_bind(
    key: &'static str,
    code: mirl::platform::KeyCode,
) -> Result<std::option::Option<std::vec::Vec<mirl::platform::KeyCode>>, String> {
    let mut keys = get_key_bind(key);
    let mut map = KEY_BINDS
        .lock()
        .map_err(|_| "Failed to lock KEY_BINDS (poisoned)".to_string())?;
    if let Some(pos) = keys.iter().position(|x| x == &code) {
    keys.remove(pos);
}
    Ok(map.insert(key, keys))
}

/// Looks up a key (immutable).
pub fn get_key_bind(key: &'static str) -> Vec<mirl::platform::KeyCode> {
    // Clone or copy depending on KeyCode; assuming it is Copy-like
    KEY_BINDS
        .lock()
        .ok()
        .and_then(|map| map.get(key).cloned()).unwrap_or_default()
}
pub fn is_key_bind_down<F:mirl::platform::framework_traits::Framework>(key: &'static str, framework: &F)->bool{
    let keys = get_key_bind(key);
    for i in keys.iter(){
        let key_down = framework.is_key_down(*i);
        if key_down{
            return true;
        }
    }
    false
}




#[inline]
fn render_block<L: Physics>(
    block: &Block,
    camera: &Camera,
    buffer: &Buffer,
    block_colors: &[u32],
    width: usize,
    height: usize,
    font: &Font,
    physics: &L,
) {
    draw_block(
        block,
        block.x.get() as isize,
        block.y.get() as isize,
        camera,
        buffer,
        block_colors[block.block_color_id],
        width,
        height,
        font,
        physics,
    );
}




pub fn get_top_most_block_idx_or_self(blocks: &[Block], index: usize) -> usize {
    let block = &blocks[index];
    if let Some(above_connection) = block.connected_above.get(){
        if let Some(idx) = index_by_block_id(&above_connection, blocks)
        {return get_top_most_block_idx_or_self(blocks, idx);}
    }
    index
}

pub fn get_bottom_most_block_idx_or_self(blocks: &[Block], index: usize) -> usize {
    let block = &blocks[index];
    if let Some(block_below) = block.connected_below.get() {
        if let Some(idx) = index_by_block_id(&block_below, blocks)
        {return get_bottom_most_block_idx_or_self(
            blocks,
            idx,
        );}
    }
    index
}

fn handle_connection_and_render_ghost_block<L: Physics, F:Framework>(
    buffer: &Buffer,
    camera: &Camera,
    blocks: &Vec<Block>,
    font: &Font,
    selected: &Option<usize>,
    snap_distance: SizeType,
    logic: &L,
    block_colors: &[u32],
    selected_type_is_action: bool,
    framework: &F
) {
    if let Some(selected) = *selected {
        if selected_type_is_action {
            // Connect to block above
            let possible_connection = logic.get_block_in_distance(
                blocks,
                blocks[selected].x.get() as SizeType,
                blocks[selected].y.get() as SizeType,
                snap_distance,
                Some(selected),
                true,
            );
            let block = &blocks[selected];
            block.possible_connection_above.set(None);

            if let Some(possible_connection) = possible_connection {
                let above_block = &blocks[possible_connection];
                if is_there_a_loop_in_block_connections_for_block_internal(
                    blocks,
                    get_top_most_block_idx_or_self(
                        blocks,
                        possible_connection,
                    ).get_id_of_idx(blocks),
                    &mut Vec::from([block.id]),
                ) {
                    debug("Skipped potential infinite loop!", framework);
                    return;
                } else {
                    // Save block, only if it is not in a loop
                    block.possible_connection_above.set(Some(above_block.id));
                }

                if !logic.is_block_visible_on_screen(
                    above_block,
                    camera,
                    &(buffer.width as isize),
                    &(buffer.height as isize),
                ) {
                    return;
                }
                draw_block(
                    block,
                    above_block.x.get() as isize,
                    above_block.y.get() as isize + block.height.get() as isize,
                    camera,
                    buffer,
                    mirl::graphics::desaturate_fast(
                        mirl::graphics::adjust_brightness_fast(
                            block_colors[block.block_color_id],
                            -5,
                        ),
                        0.91,
                    ),
                    buffer.width,
                    buffer.height,
                    font,
                    logic,
                );
            }
        } else {
            let inline_block_connection = logic.get_block_input_in_distance(
                blocks,
                blocks[selected].x.get() as SizeType,
                blocks[selected].y.get() as SizeType,
                snap_distance,
                &mirl::lists::combined(
                    &blocks[selected]
                        .get_all_ids_of_all_inputs(blocks),
                    blocks[selected].id,
                ),
                true,
            );

            if inline_block_connection.is_some() {
                println!("FOUND: {:?}", inline_block_connection)
            }
        }
    }
}

// Misc
fn handle_and_render_action_blocks_on_screen<L: Physics>(
    buffer: &Buffer,
    camera: &Camera,
    blocks: &mut Vec<Block>,
    block_colors: &[u32],
    font: &Font,
    logic: &L,
) {
    let now_width = buffer.width as isize;
    let now_height = buffer.height as isize;

    // Reverse block order in order for overdraw to to its job in our favor
    let block_ids: Vec<usize> = (0..blocks.len()).rev().collect();

    for id in block_ids {
        if blocks[id].recently_moved.get() {
            move_block_to_connected(blocks, &Some(id));
        }
        let block = &blocks[id];

        if !logic.is_block_visible_on_screen(
            block,
            camera,
            &now_width,
            &now_height,
        ) {
            continue;
        }
        render_block(
            block,
            camera,
            buffer,
            block_colors,
            buffer.width,
            buffer.height,
            font,
            logic,
        );
    }
}

#[inline]
fn reorder_element<T>(vec: &mut Vec<T>, from: usize, to: usize) {
    if from != to && from < vec.len() && to < vec.len() {
        let item = vec.remove(from);
        vec.insert(to, item)
    }
}

// Block stuff
fn get_block_id_above<L: Physics>(
    blocks: &Vec<Block>,
    pos_x: SizeType,
    pos_y: SizeType,
    logic: &L,
) -> Option<usize> {
    for (block_id, block) in blocks.iter().enumerate() {
        if logic.is_point_in_rectangle(
            pos_x,
            pos_y,
            block.x.get() as SizeType,
            block.y.get() as SizeType + block.height.get(),
            block.width.get(),
            block.height.get(),
        ) {
            return Some(block_id);
        }
    }
    None
}

#[inline]
fn get_block_id_under_point<L: Physics>(
    blocks: &Vec<Block>,
    pos_x: isize,
    pos_y: isize,
    logic: &L,
) -> Option<usize> {
    for (block_id, block) in blocks.iter().enumerate() {
        if logic.is_point_in_rectangle(
            pos_x,
            pos_y,
            block.x.get() as isize,
            block.y.get() as isize,
            block.width.get() as isize,
            block.height.get() as isize,
        ) {
            return Some(block_id);
        }
    }
    None
}

#[inline]
fn get_specific_block_id_under_point<L: Physics>(
    blocks: &Vec<Block>,
    pos_x: f64,
    pos_y: f64,
    logic: &L,
    block_type: u8,
) -> Option<usize> {
    for (block_id, block) in blocks.iter().enumerate() {
        if block.block_type != block_type {
            continue;
        }
        if logic.is_point_in_rectangle(
            pos_x,
            pos_y,
            block.x.get() as f64,
            block.y.get() as f64,
            block.width.get() as f64,
            block.height.get() as f64,
        ) {
            return Some(block_id);
        }
    }
    None
}
/// Add caching to this!!!!
/// The cache needs to be reset every frame but it'd definitely be worth it
#[inline]
pub fn index_by_block_id(id: &ID, blocks: &[Block]) -> Option<usize> {
    (0..blocks.len()).find(|&block_id| blocks[block_id].id == *id)
}

fn is_there_a_loop_in_block_connections_for_block(
    blocks: &Vec<Block>,
    block_index: ID,
) -> bool {
    let mut already_checked = Vec::new();
    is_there_a_loop_in_block_connections_for_block_internal(
        blocks,
        block_index,
        &mut already_checked,
    )
}

fn is_there_a_loop_in_block_connections_for_block_internal(
    blocks: &Vec<Block>,
    block_id: ID,
    already_checked: &mut Vec<ID>,
) -> bool {
    // I don't think this function works quite right
    if already_checked.contains(&block_id) {
        return true;
    }
    already_checked.push(block_id);
    let block = &blocks[index_by_block_id(&block_id, blocks).unwrap()];
    if let Some(connected_below) = block.connected_below.get(){
        return is_there_a_loop_in_block_connections_for_block_internal(
            blocks,
            connected_below,
            already_checked,
        );
    }
    false
}
#[inline]
fn custom_join<T: std::string::ToString>(vector: &Vec<T>, separator: &str) -> String {
    let mut out = "".to_string();
    let length = vector.len();
    for (idx, item) in vector.iter().enumerate().take(length) {
        out = out + &item.to_string();
        if idx != length - 1 {
            out += separator;
        }
    }
    out
}

#[inline]
fn get_ids_connected_below_block(
    top_most_block_id: ID,
    blocks: &Vec<Block>,
    found: &mut Vec<ID>,
) -> Vec<ID> {
    let block = &blocks[index_by_block_id(&top_most_block_id, blocks).unwrap()];

    if found.contains(&block.id) {
        panic!(
            "Infinite loop found with these ids {} with current {} (len({}))",
            custom_join(&found.to_vec(), ", "),
            top_most_block_id,
            found.len()
        )
    }
    found.push(top_most_block_id);
    if let Some(connected_below) = block.connected_below.get() {
        let sub = get_ids_connected_below_block(
            connected_below,
            blocks,
            found,
        );

        found.extend(sub)
    }

    // Filter out multiple of an id -> WE SHOULD NOT NEED TO DO THIS
    let mut seen = std::collections::HashSet::new();

    found.retain(|x| seen.insert(*x));

    found.to_vec()
}
#[inline]
fn index_by_block_ids(
    blocks: &Vec<Block>,
    ids: &Vec<ID>,
) -> Vec<Option<usize>> {
    let mut return_list = Vec::new();
    for id in ids {
        return_list.push(index_by_block_id(id, blocks))
    }
    return_list
}
fn get_total_height_of_blocks(
    blocks: &Vec<Block>,
    indexes: Vec<usize>,
) -> SizeType {
    let mut height = 0.0;
    for idx in indexes {
        let block = &blocks[idx];
        height += block.height.get();
    }
    height
}

fn get_object_out_of_option_list<T>(list: Vec<Option<T>>, skip_empty:bool) -> Vec<T> {
    let mut new = Vec::new();
    for l in list {
        if skip_empty && l.is_none(){continue;}
        new.push(l.unwrap())
    }
    new
}

fn move_block_to_connected(blocks: &mut Vec<Block>, index: &Option<usize>) {
    if index.is_none() {
        return;
    }
    let block = &blocks[index.unwrap()];
    if let Some(top_block_id) = block.connected_top.get(){
        
        let idx = match index_by_block_id(&top_block_id, blocks) {
            Some(value)=>{value}
            None=>return
        };
    let top_block =
        &blocks[idx];
    let block_query = get_ids_connected_below_block(
        top_block_id,
        blocks,
        &mut Vec::new(),
    );
    
    
    let own_block_id_index = block_query
        .iter()
        .position(|&id| id == block.id)
        .unwrap_or_else(|| 
            {

                let mut final_string = String::new();
                let mut block_names = Vec::new();
                for entry in block_query.iter(){
                    let idx = index_by_block_id(entry, blocks).unwrap();
                    let block = &blocks[idx];
                    let block_name = custom_join(&block.name, "{}");
                    block_names.push((block.id, block_name));

                }
                let mut max = 0;
                for i in block_query.iter(){
                    let num = usize::from(*i);
                    if num > max{
                        max = num;
                    }
                }
                let padding = max.ilog10();
                let mut padding_string = String::new();
                for _ in 0..padding{
                    padding_string+=" ";
                }
                for  name in block_names.iter(){
                    final_string+=&format!("ID({}{}) -> {}\n",padding_string,usize::from(name.0),name.1);
                }
                final_string+=&format!("\nNot found: ID({}) -> {}", usize::from(block.id), custom_join(&blocks[index_by_block_id(&block.id, blocks).unwrap()].name, "{}"));
                
                let mut infos = Vec::new();
                infos.push("A block thought it was inside a structure yet the structure wasn't connected to the block anymore - KEEP THE BLOCKS UPDATED".to_string());
                infos.push(format!("Structure:\n{}", final_string));
                infos.push(format!("\nBlock data: {:#?}", block));
                if block.connected_top == block.connected_above{
                    infos.push(format!("\nTopmost/Above Block data: {:#?}", top_block));
                }else{
                    infos.push(format!("\nTopmost Block data: {:#?}", top_block));
                    if let Some(connected_above) = block.connected_above.get()
                    {infos.push(format!("\nAbove Block data: {:#?}", blocks[index_by_block_id(&connected_above, blocks).unwrap()]));}

                }
                println!("{}", custom_join(&infos, "\n"));
                std::process::exit(0);});
                

    let blocks_to_offset_with =
        &block_query[0..own_block_id_index + 1].to_vec();


    let total_offset = get_total_height_of_blocks(
        blocks,
        get_object_out_of_option_list(index_by_block_ids(blocks, blocks_to_offset_with), true),
    );

    if block.recently_moved.get() {
        block.x.set(top_block.x.get());
        block.y.set(
            total_offset as CoordinateType + top_block.y.get() - block.height.get() as CoordinateType,
        );
        block.recently_moved.set(false);
    }}
}
#[inline]
fn get_difference_of_values_in_percent(value1: f64, value2: f64) -> f64 {
    if value1 == 0.0 {
        if value2 == 0.0{
            return 0.0;
        }
    return ((value2 - value1) / value1).abs() * 100.0;
    }
    ((value2 - value1) / value1).abs() * 100.0
}
fn  map_bool_to_sign<T:num_traits::One+ std::ops::Neg<Output = T>>(bool:bool)->T{
    if bool{
        T::one()
    }
    else{
        -T::one()
    }
}

fn handle_mouse_wheel<F: ExtendedFramework<f64>>(
    camera: &mut Camera,
    framework: &F,
    scroll_multiplier: f64,
    extra_scroll_multiplier: f64,
    invert_mouse_x: bool,
    invert_mouse_y:bool,
    horizontal_by_default:bool,
) {
    // Mouse wheel movement
    let mouse_wheel_temp = framework.get_mouse_scroll();
    if let Some(mut mouse_wheel_temp) = mouse_wheel_temp {
        let mut mul = scroll_multiplier;
        if horizontal_by_default{
                mouse_wheel_temp = (mouse_wheel_temp.1, mouse_wheel_temp.0)

        }
            if is_key_bind_down("horizontal_scroll", framework) {
                mouse_wheel_temp = (mouse_wheel_temp.1, mouse_wheel_temp.0)
            }
            if is_key_bind_down("scroll_multiply", framework)  {
                mul*=extra_scroll_multiplier
            }

            
        if is_key_bind_down("modify_camera_z", framework) {
            camera.z -= mouse_wheel_temp.1 * mul;
        } else {
            camera.x -= -((mouse_wheel_temp.0
                * mul) as isize*map_bool_to_sign::<isize>(invert_mouse_x));
            camera.y -= -((mouse_wheel_temp.1
                * mul) as isize*map_bool_to_sign::<isize>(invert_mouse_y));
        }
    }
}
fn reorder_blocks(
    blocks: &mut Vec<Block>,
    selected: &mut usize,
) {
            reorder_element(blocks, *selected, 0);
            *selected = 0;
        }
fn handle_mouse_or_get_selected<
    F: mirl::platform::framework_traits::Framework,
    L: Physics,
>(
    mouse_down: bool,
    last_mouse_down: bool,
    blocks: &mut Vec<Block>,
    camera: &mut Camera,
    mouse_pos: (isize, isize),
    // mouse_delta: (f32, f32),
    framework: &F,
    // scroll_multiplier: f32,
    selected: Option<usize>,
    logic: &L,
    _selected_type_is_action: bool,
    cursor_style: &mut Option<mirl::platform::Cursor>,
    cursors: &mut mirl::platform::mouse::Cursors,
) -> Option<usize> {
    // There are too many problems with dealing with null when the mouse is outside the window, so instead we just check if the mouse is with in the window :)
    let mut selected = selected;
    let under_mouse = get_block_id_under_point(
        blocks,
        mouse_pos.0 + camera.x,
        mouse_pos.1 + camera.y,
        logic,
    );
    if under_mouse.is_some() {
        if mouse_down {
            *cursor_style = Some(cursors.closed_hand);
        } else {
            *cursor_style = Some(cursors.open_hand);
        }
    } else {
        *cursor_style = Some(cursors.default)
    }

    if mouse_down {
        // No clue what this is about but it works so I'm not gonna change it
        if !last_mouse_down {
            selected = under_mouse;
        }
        if let Some(selected) = selected {
            *cursor_style = Some(cursors.closed_hand);
            // When this block is selected, disconnect it from the blocks above
            let selected_block = &blocks[selected];
            
            // If ctrl is pressed, the structure below should be connected with the structure above
            let disconnect_single = is_key_bind_down("select_single", framework);
            let duplicate =is_key_bind_down("duplicate", framework);
            if disconnect_single&&duplicate{
                // Only copy the singular block
            }else if duplicate{
                // Copy entire block structure downwards
            }
            else if disconnect_single{
                selected_block.connect_below_to_above(blocks);
            }else{
            selected_block.disconnect_above(blocks);
            selected_block.update_topmost(blocks, true);
        }
        }
    } else {
        // Connect block previously selected if possible
        if let Some(selected)=selected {
            let block = &blocks[selected];
            if block.block_type == 1 {
                return None;
            }
            if block.possible_connection_above.get().is_some() {
                // let connection_id_above =
                //     block.possible_connection_above.get().unwrap();

                if is_there_a_loop_in_block_connections_for_block_internal(
                    blocks,
                    get_top_most_block_idx_or_self(
                        blocks,
                        index_by_block_id(
                            &block.possible_connection_above.get().unwrap(),
                            blocks,
                        )
                        .unwrap(),
                    ).get_id_of_idx(blocks),
                    &mut Vec::from([blocks[selected].id]),
                ) {
                    panic!("Loop detected in block structure");
                }
                block.connect_to_possibly_above(blocks);
                
                // // Get above block
                // let block_above_index =
                //     index_by_block_id(&connection_id_above, blocks).unwrap();
                // let above_block = &blocks[block_above_index];
                // if let Some(block_below_above) =
                //     above_block.connected_below.get()
                // {
                //     let self_bottom_most_id = get_bottom_most_block_id_or_self(
                //         blocks,
                //         index_by_block_id(&block.id, blocks).unwrap(),
                //     );
                //     let self_bottom_most = &blocks[index_by_block_id(
                //         &self_bottom_most_id,
                //         blocks,
                //     )
                //     .unwrap()];
                //     self_bottom_most
                //         .connected_below
                //         .set(Some(block_below_above));
                // }

                // // Tell current block to connect above
                // // block.connected_top.set(Some(get_top_most_block_id_or_self(
                // //     blocks,
                // //     block_above_index,
                // // )));
                // block.connected_above.set(Some(above_block.id));
                // above_block.connected_below.set(Some(block.id));
                block.update_topmost(blocks, true);
            }

            // Set current possible above block to none
            block.possible_connection_above.set(None);
            //block.recently_moved.set(true); // Did commenting this out do anything?
        }
        selected = None;
    }

    selected
}


fn load_blocks<F: FileSystem, L: Physics>(
    file_system: &F,
    block_output_types: &mut Vec<String>,
    block_output_colors: &mut Vec<u32>,
    font: &Font,
    workspace: &mut WorkSpace<L>,
) -> (Vec<Block>, Vec<Block>, HashMap<String, String>) {
    let path = r"C:\personal\games\minecraft\Automated\ender_py\procedures";
    let mut action_blocks = Vec::new();
    let mut inline_blocks = Vec::new();
    let fallback_language = "en";
    let mut translation = HashMap::new();
    let all_plugin_folders = file_system.get_folders_in_folder(path);
    
    
    let mut errors: Vec<String> = Vec::new();
    for plugin_folder in all_plugin_folders {
        let plugin_path = file_system.join(path, &plugin_folder);
        let setting_file = file_system.join(&plugin_path, "settings.json");
        let translation_file =
            file_system.join(&plugin_path, "translation.csv");
        
        
        let translation_data =
            file_system.get_file_contents(&translation_file).unwrap();
        let (_used_language, parsed_translation) =
            crate::internal::svg::parse_translations(&translation_data.as_string().unwrap(), fallback_language,fallback_language)
                .unwrap();
        translation = parsed_translation;


        let settings_json: serde_json::Value = serde_json::from_str(
            &file_system
                .get_file_contents(&setting_file)
                .unwrap()
                .as_string()
                .unwrap(),
        )
        .expect("Unable to load json settings file");
        let pre_blocks = settings_json
            .get("blocks")
            .ok_or("Error")
            .expect("Error unwrapping");
        for block in
            pre_blocks.as_array().ok_or("Error").expect("Error unwrapping")
        {
            // framework.log(
            //     "{}",
            //     block.get("name").ok_or("Error").expect("Error unwrapping")
            // );
            let block_type =
                block.get("type").ok_or("Error").expect("Error unwrapping");
            if block_type == "action" || block_type == "inline" {
                let internal_name: String = block
                    .get("name")
                    .expect("Missing name")
                    .as_str()
                    .expect("Error unwrapping")
                    .to_string();

                let output;
                if let Some(_output) = block.get("output") {
                    if let Some(__output) = _output.as_str() {
                        output = __output.to_string()
                    } else {
                        errors.push(format!(
                            "Output of {} isn't a string",
                            internal_name
                        ));
                        output = "You aren't supposed to read me".into();
                    }
                } else {
                    errors
                        .push(format!("Missing output for {}", internal_name));
                    output = "You aren't supposed to read me either".into();
                }

                if !block_output_types.contains(&output) {
                    block_output_types.push(output.clone());
                    block_output_colors.push(generate_random_color());
                }
                let name: String;
                if let Some(_name) = translation.get(&internal_name) {
                    name = (*_name).clone()
                } else {
                    errors.push(format!(
                        "Translation key '{}' not found",
                        internal_name
                    ));
                    name = internal_name.clone()
                }

                let _empty = serde_json::Value::Array(Vec::new());
                let pre_inputs = block
                    .get("inputs")
                    .unwrap_or(&_empty)
                    .as_array()
                    .expect("Error unwrapping");

                let mut inputs = Vec::new();
                for pre_input in pre_inputs {
                    let temp = pre_input.as_object().unwrap();
                    let key_and_return_temp;
                    let in_case_of_literal_keys_and_return_values =
                        temp.get("expected");

                    let mut expected_literal_allowed = Vec::new();
                    let mut expected_literal_return = Vec::new();

                    if in_case_of_literal_keys_and_return_values.is_some() {
                        key_and_return_temp =
                            in_case_of_literal_keys_and_return_values
                                .expect("THIS SHOULD LITERALLY BE IMPOSSIBLE")
                                .as_object();
                        //framework.log("{:?}", key_and_return_temp);

                        let keys = key_and_return_temp.expect("Expected key 'expected' to be a dict, are you sure you didn't accidentally make it a list?").keys();
                        for key in keys {
                            let value =
                                key_and_return_temp.unwrap().get(key).unwrap();
                            expected_literal_allowed.push(key.clone());
                            expected_literal_return
                                .push(value.as_str().unwrap().to_string());
                        }
                    }

                    let input = BlockInput::new(
                        temp.get("type").unwrap().as_str().unwrap().to_string(),
                        None,
                        expected_literal_allowed,
                        expected_literal_return,
                    )
                    .unwrap();
                    inputs.push(input);
                }
                //framework.log(">{:?}", pre_inputs);

                let inline = block_type == "inline";
                let block_type_id = if inline {
                    1
                } else {
                    0
                };
                if !errors.is_empty() {
                    continue;
                }

                let block = Block::new(
                    name.clone(),
                    internal_name,
                    0,
                    0,
                    block_type_id,
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    output,
                    inputs,
                    block_output_types,
                    font,
                    workspace,None
                );
                if inline {
                    inline_blocks.push(block);
                } else {
                    action_blocks.push(block);
                }
            } else {
                //framework.log("Skipping Block as {} is not supported", block_type);
                continue;
            }
        }
    }

    if !errors.is_empty() {
        panic!(
            "Unable to load plugins due to the following error(s):\n{:#?}",
            errors
        );
    }

    (action_blocks, inline_blocks, translation)
}
#[inline]
pub fn generate_random_color() -> u32 {
    getrandom::u32().expect("Unable to generate random color")
}

// #[inline]
// fn title_case(s: &str) -> String {
//     s.split_whitespace()
//         .map(|word| {
//             let mut chars = word.chars();
//             match chars.next() {
//                 Some(first) => {
//                     first.to_uppercase().collect::<String>() + chars.as_str()
//                 }
//                 None => String::new(),
//             }
//         })
//         .collect::<Vec<_>>()
//         .join(" ")
// }
// fn internal_name_to_name(internal_name: &str) -> String {
//     let mut name = internal_name.to_string();
//     name = name.replace("_", " ");
//     //title case
//     name = title_case(&name);
//     return name;
// }
#[inline(always)]
fn debug(string: &str, framework:&dyn mirl::platform::framework_traits::Framework) {
    if super::DEBUG {
        framework.log(string);
    }
}

// Goal: Render 1000 Blocks at >=60 fps
// Current:
// Optimized (10_000):
// FAST_RENDER OFF: 18 fps
// FAST_RENDER ON: 25 fps

pub fn main_loop<
    F: ExtendedFramework<f64>,
    D: mirl::platform::FileSystem,
    // S: RenderSettings,
    // L: Physics,
>(
    framework: &mut F,
    file_system: &D,
    // render_settings: &S,
    // logic: &L,
    buffer: &Buffer,
)-> Result<(), Box<dyn std::error::Error>> {
    debug("Setting Default Key-binds", framework);
    add_key_bind("single_select", mirl::platform::KeyCode::LeftControl)?;
    add_key_bind("single_select", mirl::platform::KeyCode::RightControl)?;
    add_key_bind("duplicate", mirl::platform::KeyCode::LeftAlt)?;
    add_key_bind("duplicate", mirl::platform::KeyCode::RightAlt)?;
    add_key_bind("debug_block", mirl::platform::KeyCode::I)?;
    add_key_bind("next_workspace", mirl::platform::KeyCode::Right)?;
    add_key_bind("previous_workspace", mirl::platform::KeyCode::Left)?;
    add_key_bind("horizontal_scroll", mirl::platform::KeyCode::LeftShift)?;
    add_key_bind("scroll_multiply", mirl::platform::KeyCode::RightShift)?;
    add_key_bind("modify_camera_z", mirl::platform::KeyCode::LeftControl)?;
    add_key_bind("modify_camera_z", mirl::platform::KeyCode::RightControl)?;
    
    
    debug("Setting up physics and workspace list", framework);
    let fast_physics = crate::logic::LogicFast::new();
    let accurate_physics = crate::logic::LogicFast::new();
    let mut workspaces =
        Vec::from([WorkSpace::new(&accurate_physics), WorkSpace::new(&fast_physics)]);
    // : Vec<WorkSpace<'_, dyn Physics>>
    let mut current_workspace_id = 0;
    let workspace_length = workspaces.len();

    let mut current_workspace = &mut workspaces[current_workspace_id];
    debug("Setting window icon", framework);
    let icon: Buffer = file_system
        .get_file_contents("idk.ico")
        .unwrap()
        .as_image()
        .unwrap()
        .into();

    framework.set_icon(&icon.data, icon.width as u32, icon.height as u32);


    debug("Setting default variables", framework);
    // Settings - Do touch!
    // Settings - Modifiable after launch
    let snap_distance = 70.0;
    let scroll_multiplier = 5.0;
    let extra_scroll_multiplier = 5.0;
    let max_fps = 60;
    let invert_mouse_wheel_x = false;
    let invert_mouse_wheel_y = false;
    let horizontal_mouse_wheel_by_default = false;
    // Settings - Modifiable before launch
    let custom_main_cursor_color = (100, 20, 250);
    let custom_secondary_cursor_color = (80, 30, 240);

    // Setting - Do not touch!

    framework.set_target_fps(max_fps);
    //let target_frame_delta = mirl::time::MILLIS_PER_SEC / max_fps; // Time for one frame at the target FPS
    let mut frame_start;

    let mut delta_time;
    let mut fps;

    let mut mouse_pos = framework.get_mouse_position().unwrap_or((0, 0));
    let mut mouse_delta;
    let mut mouse_held;
    let mut mouse_down_temp;
    let mut mouse_down = false;
    let mut mouse_outside;

    let mut block_output_color_rgb: Vec<u32> = Vec::new();
    let mut block_output_color_names: Vec<String> = Vec::new();

    let mut fps_list: Vec<u64> = Vec::new();

    let mut selected: Option<usize> = None;
    let mut selected_type_is_action: bool = false;


    let mut next_workspace_key_down = false;
    let mut previous_workspace_key_down = false;

    debug("Loading custom cursors", framework);
    let cursors = framework.load_custom_cursor(
        U2::new(0),
        rgb_to_u32(custom_main_cursor_color.0,custom_main_cursor_color.1,custom_main_cursor_color.2),
        rgb_to_u32(custom_secondary_cursor_color.0,custom_secondary_cursor_color.1,custom_secondary_cursor_color.2),
    );
    debug("Loading Font, translations, blocks, and colors", framework);
    let font =
        file_system.get_file_contents("inter.ttf").unwrap().as_font().unwrap();


    let (action_blocks, inline_blocks, _translation) = load_blocks(
        file_system,
        &mut block_output_color_names,
        &mut block_output_color_rgb,
        &font,
        current_workspace,
    );
    current_workspace.blocks.extend(action_blocks.iter().cloned());
    current_workspace.blocks.extend(inline_blocks.iter().cloned());

    frame_start = framework.get_time();

    while framework.is_open() {
        debug("Start of loop", framework);
        buffer.clear();

        let mut cursor_style: Option<mirl::platform::Cursor> = None;

        debug("Checking for change in workspace", framework);
        let mut reload_workspace = false;
        if is_key_bind_down("next_workspace", framework)
            && !next_workspace_key_down
            && workspace_length > current_workspace_id + 1
        {
            current_workspace_id += 1;
            next_workspace_key_down = true;
            reload_workspace = true
        } else {
            next_workspace_key_down = false
        }
        if is_key_bind_down("previous_workspace", framework)
            && !previous_workspace_key_down
            && current_workspace_id > 0
        {
            current_workspace_id -= 1;
            reload_workspace = true;
            previous_workspace_key_down = true
        } else {
            previous_workspace_key_down = false
        }
        if reload_workspace {
            current_workspace = &mut workspaces[current_workspace_id];
            if current_workspace.blocks.is_empty() {
                current_workspace.blocks.extend(action_blocks.iter().cloned());
                current_workspace.blocks.extend(inline_blocks.iter().cloned())
            }
        }

        debug("Getting and handling mouse info", framework);
        // Mouse stuff and block(/camera) selection/movement
        mouse_delta = mouse_pos;
        mouse_pos = framework.get_mouse_position().unwrap_or(mouse_pos);

        mouse_delta = mouse_pos.sub(mouse_delta);

        // mirl::render::draw_circle(
        //     buffer,
        //     mouse_pos.0 as usize,
        //     mouse_pos.1 as usize,
        //     10,
        //     rgb_to_u32(255, 0, 0),
        //     true,
        // );
        // println!(
        //     "{:?} | {:.3?} | {:?}",
        //     raw_mouse_delta,
        //     mouse_delta,
        //     raw_mouse_delta.sub(mouse_delta.tuple_2_into())
        // );
        // mirl::render::draw_circle(
        //     buffer,
        //     (mouse_pos.0 + raw_mouse_delta.0 as isize) as usize,
        //     (mouse_pos.1 + raw_mouse_delta.1 as isize) as usize,
        //     10,
        //     rgb_to_u32(0, 0, 255),
        //     true,
        // );

        mouse_outside = !current_workspace.logic.is_point_in_rectangle(
            mouse_pos.0,
            mouse_pos.1,
            0,
            0,
            buffer.width as isize,
            buffer.height as isize,
        );

        mouse_down_temp = mouse_down;
        mouse_down = framework.is_mouse_down(mirl::platform::MouseButton::Left);
        mouse_held = mouse_down_temp && mouse_down;

        handle_mouse_wheel(
            &mut current_workspace.camera,
            framework,
            scroll_multiplier,extra_scroll_multiplier,invert_mouse_wheel_x,invert_mouse_wheel_y,horizontal_mouse_wheel_by_default
        );

        if !mouse_outside {
            debug("Getting new selected", framework);
            selected = handle_mouse_or_get_selected(
                mouse_down,
                mouse_held,
                &mut current_workspace.blocks,
                &mut current_workspace.camera,
                mouse_pos,
                // mouse_delta,
                framework,
                // scroll_multiplier,
                selected,
                current_workspace.logic,
                selected_type_is_action,
                &mut cursor_style,
                &mut cursors.clone(),
            );
        }
        if selected.is_some() && !mouse_held {
            selected_type_is_action =
                current_workspace.blocks[selected.unwrap()].block_type == 0;
        }

        if mouse_down {
            if selected.is_some() {
                //framework.log("SELECTED");
                cursor_style = Some(cursors.closed_hand);
            } else {
                cursor_style = Some(cursors.all_scroll);
                //framework.log("UNSELECTED");
            }
        } else {
            //framework.set_cursor_style(&cursors.default);
        }


        //framework.log("Selected: {:?}, {}", selected, selected_type_is_action);
        debug("Reordering block structure", framework);
        
    if mouse_down {
        if let Some(idx) = &mut selected {
        reorder_blocks(
            &mut current_workspace.blocks,
            idx,
        );
        let selected_block = &mut current_workspace.blocks[0];
    
            selected_block.x
                .set((selected_block.x.get() as isize + mouse_delta.0) as CoordinateType);
            selected_block.y
                .set((selected_block.y.get() as isize + mouse_delta.1) as CoordinateType);
    
    } else {
            current_workspace.camera.x -= mouse_delta.0;
            current_workspace.camera.y -= mouse_delta.1;
        }
    }


        if selected.is_some() && selected.unwrap() != 0 {
            panic!("Selected is not 0 -> Reordering failed?");
        }
        //############################################
        debug("Handling/Rendering blocks on screen", framework);
        handle_and_render_action_blocks_on_screen(
            buffer,
            &current_workspace.camera,
            &mut current_workspace.blocks,
            &block_output_color_rgb,
            &font,
            current_workspace.logic,
        );
        // handle_and_render_action_blocks_on_screen(
        //     buffer,
        //     &current_workspace.camera,
        //     &mut current_workspace.blocks,
        //     &block_output_color_rgb,
        //     &font,
        //     current_workspace.logic,
        // );
        debug("Handling Connection/Handle Ghost block", framework);
        handle_connection_and_render_ghost_block(
            buffer,
            &current_workspace.camera,
            &current_workspace.blocks,
            &font,
            &selected,
            snap_distance,
            current_workspace.logic,
            &block_output_color_rgb,
            selected_type_is_action, framework
        );

        if is_key_bind_down("debug_block", framework){
            framework.log("\nINFOS\n");
            if let Some(selected_block_index) = selected{
                let all_connected = get_ids_connected_below_block(current_workspace.blocks[selected_block_index].id, &current_workspace.blocks, &mut Vec::new());
                
                for id in all_connected{
                framework.log(&format!("Selected Block info:\n{:#?}", current_workspace.blocks[index_by_block_id(&id, &current_workspace.blocks).unwrap()]));

                }
            }else if let Some(under_mouse) = get_block_id_under_point(&current_workspace.blocks, mouse_pos.0, mouse_pos.1, current_workspace.logic){
                let all_connected = get_ids_connected_below_block(current_workspace.blocks[under_mouse].id, &current_workspace.blocks, &mut Vec::new());
                
                for id in all_connected{
                framework.log(&format!("Under Mouse Block info:\n{:#?}", current_workspace.blocks[index_by_block_id(&id, &current_workspace.blocks).unwrap()]));

        }
            }
            else
            {
                
                for block in current_workspace.blocks.iter(){
                framework.log(&format!("All Block info:\n{:#?}", block));

                }
            }
        }

        //############################################
        debug("Update framework with buffer", framework);
        framework.update(buffer);
        debug("Calculating FPS", framework);
        delta_time = frame_start.get_elapsed_time();
        frame_start = framework.get_time();

        if delta_time != 0.0 {
            fps = 1.0 / delta_time;
        } else {
            fps = f64::MAX;
        }
        let average_fps = fps_list.average().unwrap_or_default();

        add_item_to_max_sized_list(
            &mut fps_list,
            average_fps.max(1) as usize,
            fps as u64,
        );

        if let Some(cursor) = cursor_style {
            debug("Setting cursors style", framework);
            framework.set_cursor_style(&cursor);
        }

        debug("Updating title", framework);
        framework.set_title(
            &to_monospace_unicode(&format!(
                "Rust Window {:>4}/{:>5.0} FPS (Sampling {:>3}) | {:>8}x {:>8}y {:>4}z | {:>8}x {:>8}y -> {:>4} {:>4} -> {:>3} {:>3} | {}A + {}I = {}T",
                average_fps,
                fps,
                fps_list.len(),
                current_workspace.camera.x,
                current_workspace.camera.y,
                current_workspace.camera.z,
                current_workspace.camera.x + mouse_pos.0 ,
                current_workspace.camera.y + mouse_pos.1 ,
                mouse_pos.0,
                mouse_pos.1,
                mouse_delta.0,
                mouse_delta.1,
                current_workspace.blocks.len(),
                current_workspace.blocks.len(),
                current_workspace.blocks.len()+
                current_workspace.blocks.len()
            ))
        );
        // if selected.is_some() {
        //     let block = &blocks[selected.unwrap()];
        //     framework.log("Block Index: {}, ID: {}, Connected bottom: {}, Connected top: {}, Possible connection below: {}, Possible Connection Above: {}",
        //     selected.unwrap(), block.id,
        //     block.connected_below.get().map_or("None".to_string(), |id| id.to_string()),
        //     block.connected_top.get().map_or("None".to_string(), |id| id.to_string()),
        //     block.possible_connection_below.get().map_or("None".to_string(), |id| id.to_string()),
        //     block.possible_connection_above.get().map_or("None".to_string(), |id| id.to_string()))
        // }

        // if delta_time < target_frame_delta && false {
        //     let sleep_time = target_frame_delta - delta_time;
        //     // platform::log(&format!(
        //     //     "Delta: {}, Needed: {}, Sleeping for: {}",
        //     //     delta_time, target_frame_delta, sleep_time
        //     // ));
        //     framework.wait(sleep_time);
        // }
        debug("End of loop", framework);
    }
    Ok(())
}

fn to_monospace_unicode(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            // Digits 0–9
            '0'..='9' => char::from_u32(0x1D7F6 + (c as u32 - '0' as u32)),
            // Uppercase A–Z
            'A'..='Z' => char::from_u32(0x1D670 + (c as u32 - 'A' as u32)),
            // Lowercase a–z
            'a'..='z' => char::from_u32(0x1D68A + (c as u32 - 'a' as u32)),
            // Fallback: return original char
            _ => Some(c),
        })
        .map(|c| c.unwrap_or('�'))
        .collect()
}
// fn main() {
//     let width = 800;
//     let height = 600;

//     let window_name = "Rust window";
//     // Create a window with the title "Rust Window"
//     let mut window = Window::new(
//         &window_name, // Window title
//         width,        // Width
//         height,       // Height
//         WindowOptions {
//             title: true,
//             ..WindowOptions::default()
//         },
//     )
//     .expect("Unable to create window");

//     #[cfg(target_os = "windows")]
//     window.set_icon(Icon::from_str("src/idk.ico").unwrap());
// }
