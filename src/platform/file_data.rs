use image::GenericImageView;

pub struct FileData {
    raw_data: Vec<u8>,
}

impl FileData {
    // Constructor to load data from raw bytes
    pub fn from_bytes(data: Vec<u8>) -> Self {
        FileData {
            raw_data: data,
        }
    }

    // Convert the raw bytes to a String (if valid UTF-8)
    pub fn as_string(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.raw_data.clone())
    }

    // Convert the raw bytes to a Number (assumes data is in a binary format like little-endian)
    pub fn as_number(&self) -> Result<i64, std::io::Error> {
        if self.raw_data.len() < 8 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Not enough data",
            ));
        }
        let number =
            i64::from_le_bytes(self.raw_data[0..8].try_into().unwrap());
        Ok(number)
    }

    pub fn as_image(
        &self,
    ) -> Result<(Vec<u32>, u32, u32), Box<dyn std::error::Error>> {
        // Decode the raw bytes as an image
        let img = image::load_from_memory(&self.raw_data)?;

        // Get the pixel data
        let (width, height) = img.dimensions();
        let rgba_img = img.to_rgba8();

        let mut new = Vec::new();

        for y in 0..height {
            for x in 0..width {
                let pixel = rgba_img.get_pixel(x, y);
                new.push(mirl::graphics::rgba_to_u32(*pixel));
            }
        }

        Ok((new, width, height))
    }
    // Get raw bytes
    pub fn as_bytes(&self) -> &Vec<u8> {
        &self.raw_data
    }
}
