pub struct Camera {
    pub x: isize,
    pub y: isize,
    pub z: f32,
}

impl Camera {
    pub fn new() -> Self {
        return Camera {
            x: (u16::MAX / 2) as isize,
            y: (u16::MAX / 2) as isize,
            z: 1.0,
        };
    }
}
