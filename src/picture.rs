use std::convert::From;
use std::i32;

use crate::mesh::Vec4;

#[derive(Debug, PartialEq)]
pub struct RGBA {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl From<(u8, u8, u8, u8)> for RGBA {
    fn from(rgba: (u8, u8, u8, u8)) -> Self {
        Self {
            r: rgba.0,
            g: rgba.1,
            b: rgba.2,
            a: rgba.3,
        }
    }
}

impl From<(f32, f32, f32, f32)> for RGBA {
    fn from(rgba: (f32, f32, f32, f32)) -> Self {
        Self {
            r: (rgba.0.min(1.0).max(0.0) * 255.0) as u8,
            g: (rgba.1.min(1.0).max(0.0) * 255.0) as u8,
            b: (rgba.2.min(1.0).max(0.0) * 255.0) as u8,
            a: (rgba.3.min(1.0).max(0.0) * 255.0) as u8,
        }
    }
}

impl From<&str> for RGBA {
    fn from(rgba: &str) -> Self {
        assert_eq!(rgba.len(), 8);

        Self {
            r: i32::from_str_radix(&rgba[0..2], 16).unwrap() as u8,
            g: i32::from_str_radix(&rgba[2..4], 16).unwrap() as u8,
            b: i32::from_str_radix(&rgba[4..6], 16).unwrap() as u8,
            a: i32::from_str_radix(&rgba[6..8], 16).unwrap() as u8,
        }
    }
}

impl From<&Vec4> for RGBA {
    fn from(vec: &Vec4) -> Self {
        Self {
            r: (vec.x.min(1.0).max(0.0) * 255.0) as u8,
            g: (vec.y.min(1.0).max(0.0) * 255.0) as u8,
            b: (vec.z.min(1.0).max(0.0) * 255.0) as u8,
            a: (vec.w.min(1.0).max(0.0) * 255.0) as u8,
        }
    }
}

#[derive(Debug)]
pub struct Picture {
    data: Vec<u8>,
    width: usize,
    height: usize,
    depth: usize,
}

impl Picture {
    pub fn new(width: usize, height: usize) -> Self {
        let depth = 4;
        let mut data = Vec::new();
        data.resize((width * height * depth) as usize, 0);

        let mut pic = Picture {
            data,
            width,
            height,
            depth,
        };

        pic.fill(&(0, 0, 0, 255).into());
        pic
    }

    pub fn stride(&self) -> usize {
        self.width * self.depth
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    pub fn to_bgra(&self) -> Vec<u8> {
        let mut bgra = Vec::<u8>::with_capacity(self.data.len());
        for i in (0..self.data.len()).step_by(4) {
            bgra.push(self.data[i+2]);
            bgra.push(self.data[i+1]);
            bgra.push(self.data[i+0]);
            bgra.push(self.data[i+3]);
        }

        bgra
    }

    pub fn data_as_boxed_slice(&mut self) -> Box<[u8]> {
        self.data.clone().into_boxed_slice()
    }

    pub fn fill(&mut self, rgba: &RGBA) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.set(x, y, rgba.into());
            }
        }
    }

    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, rgba: &RGBA) {
        // Bresenham's line algorithm
        let mut x = x0;
        let mut y = y0;

        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 as i32 - y0 as i32).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.set(x as usize, y as usize, rgba);
            if x == x1 && y == y1 {
                break;
            }

            let e2 = 2 * err;
            if e2 > dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    pub fn set(&mut self, x: usize, y: usize, rgba: &RGBA) {
        if x >= self.width || y >= self.height {
            return;
        }

        let stride = self.stride();
        self.data[stride * y + (x * self.depth) + 0] = rgba.r;
        self.data[stride * y + (x * self.depth) + 1] = rgba.g;
        self.data[stride * y + (x * self.depth) + 2] = rgba.b;
        self.data[stride * y + (x * self.depth) + 3] = rgba.a;
    }

    pub fn get(&self, x: usize, y: usize) -> RGBA {
        let stride = self.stride();
        (
            self.data[stride * y + (x * self.depth) + 0],
            self.data[stride * y + (x * self.depth) + 1],
            self.data[stride * y + (x * self.depth) + 2],
            self.data[stride * y + (x * self.depth) + 3],
        )
            .into()
    }

    pub fn test_pattern(&mut self) {
        for i in 0..self.height.min(self.width) {
            self.set(i, i, &(255, 0, 0, 0).into());
        }
    }

    pub fn save(&self, path: &str) -> std::io::Result<()> {
        let file = std::fs::File::create(path)?;
        let buf = std::io::BufWriter::new(file);
        let mut encoder = png::Encoder::new(buf, self.width as u32, self.height as u32);

        encoder.set_color(png::ColorType::RGBA);
        encoder.set_depth(png::BitDepth::Eight);

        let mut writer = encoder.write_header()?;
        writer.write_image_data(&self.data)?;

        Ok(())
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_rgba() {
        let rgba: RGBA = (1.0, 0.5, -1.0, 2.0).into();
        assert_eq!(rgba, (255, 127, 0, 255).into());

        let rgba: RGBA = "FF00FF00".into();
        assert_eq!(rgba, (255, 0, 255, 0).into());
    }
}
