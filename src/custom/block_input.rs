// use crate::all::get_length_of_text_in_font;

#[derive(Clone, Debug, PartialEq)]
pub struct BlockInput {
    input_type: String,
    block_id: Option<usize>,
    literal_allowed: Vec<String>,
    literal_return: Vec<String>,
}
impl BlockInput {
    pub fn new(
        input_type: String,
        block_id: Option<usize>,
        literal_allowed: Vec<String>,
        literal_return: Vec<String>,
    ) -> Result<Self, &'static str> {
        if literal_allowed.len() != literal_return.len() {
            return Err(
                "expected and expected_return must have the same length",
            );
        }

        Ok(Self {
            block_id,
            input_type,
            literal_allowed,
            literal_return,
        })
    }
    pub fn get_width(&self, _font: &fontdue::Font) -> f32 {
        if self.block_id.is_none() {
            return 20.0;
        }
        10.0
        //get_length_of_text_in_font(&self.name, font)
    }
}
