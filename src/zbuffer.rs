pub struct ZBuffer {
    data: Vec<f32>,
    width: usize,
    height: usize,
}

impl ZBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let mut data = Vec::new();
        data.resize((width * height) as usize, f32::MIN);

        Self { data, width, height }
    }

    pub fn test_and_set(&mut self, x: usize, y: usize, z: f32) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }

        if z > self.data[y * self.width + x] {
            self.data[y * self.width + x] = z;
            return true;
        }

        false
    }
}
