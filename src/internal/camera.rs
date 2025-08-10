use crate::CoordinateType;

pub struct Camera {
    pub x: isize,
    pub y: isize,
    pub z: f64,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            x: (CoordinateType::MAX / 2) as isize,
            y: (CoordinateType::MAX / 2) as isize,
            z: 1.0,
        }
    }
}
