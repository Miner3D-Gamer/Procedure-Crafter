pub struct BlockInput {
    input_type: String,
    name: String,
    expected: Option<Vec<String>>,
    expected_return: Option<Vec<String>>,
}
impl BlockInput {
    fn new(
        input_type: String,
        name: String,
        expected: Option<Vec<String>>,
        expected_return: Option<Vec<String>>,
    ) -> Result<Self, &'static str> {
        if let (Some(ref e), Some(ref er)) = (&expected, &expected_return) {
            if e.len() != er.len() {
                return Err(
                    "expected and expected_return must have the same length",
                );
            }
        }

        Ok(Self {
            input_type,
            name,
            expected,
            expected_return,
        })
    }
}
