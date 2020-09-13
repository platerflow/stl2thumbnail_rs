use std::convert::From;
use std::i32;

use crate::mesh::Vec4;
use gif::SetParameter;
use std::mem::swap;

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
            bgra.push(self.data[i + 2]);
            bgra.push(self.data[i + 1]);
            bgra.push(self.data[i + 0]);
            bgra.push(self.data[i + 3]);
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

    pub fn line_aa(&mut self, mut x0: i32, mut y0: i32, mut x1: i32, mut y1: i32, rgba: &RGBA) {
        // Xiaolin Wu's line algorithm
        // https://en.wikipedia.org/wiki/Xiaolin_Wu%27s_line_algorithm
        fn ipart(x: f32) -> f32 {
            x.floor()
        }

        fn round(x: f32) -> f32 {
            ipart(x + 0.5)
        }

        fn fpart(x: f32) -> f32 {
            x - x.floor()
        }

        fn rfpart(x: f32) -> f32 {
            1.0 - fpart(x)
        }

        fn plot(pic: &mut Picture, x: f32, y: f32, c: f32, rgba: &RGBA) {
            pic.set(
                x as usize,
                y as usize,
                &(
                    (rgba.r as f32 * c) as u8,
                    (rgba.r as f32 * c) as u8,
                    (rgba.r as f32 * c) as u8,
                    (rgba.r as f32 * c) as u8,
                )
                    .into(),
            );
        }

        let steep = (y1 - y0).abs() > (x1 - x0).abs();

        if steep {
            swap(&mut x0, &mut y0);
            swap(&mut x1, &mut y1);
        } else if x0 > x1 {
            swap(&mut x0, &mut x1);
            swap(&mut y0, &mut y1);
        }

        let mut dx = (x1 - x0) as f32;
        let mut dy = (y1 - y0) as f32;

        let mut grad = dy / dx;
        if dx == 0.0 {
            grad = 1.0;
        }

        // first endpoint
        let xend = round(x0 as f32);
        let yend = y0 as f32 + grad * (xend - x0 as f32);
        let xgap = rfpart(x0 as f32 + 0.5);
        let mut xpxl1 = xend;
        let mut ypxl1 = ipart(yend);
        if steep {
            plot(self, ypxl1, xpxl1, rfpart(yend) * xgap, rgba);
            plot(self, ypxl1 + 1.0, xpxl1, fpart(yend) * xgap, rgba)
        } else {
            plot(self, xpxl1, ypxl1, rfpart(yend) * xgap, rgba);
            plot(self, xpxl1, ypxl1 + 1.0, fpart(yend) * xgap, rgba);
        }

        // first y-intersection for the main loop
        let mut intery = yend + grad;

        // second endpoint
        let xend = round(x1 as f32);
        let yend = y1 as f32 + grad * (xend - x1 as f32);
        let xgap = fpart(x1 as f32 + 0.5);
        let xpxl2 = xend;
        let ypxl2 = ipart(yend);
        if steep {
            plot(self, ypxl2, xpxl2, rfpart(yend) * xgap, rgba);
            plot(self, ypxl2 + 1.0, xpxl2, fpart(yend) * xgap, rgba);
        } else {
            plot(self, xpxl2, ypxl2, rfpart(yend) * xgap, rgba);
            plot(self, xpxl2, ypxl2 + 1.0, fpart(yend) * xgap, rgba);
        }

        // main loop
        if steep {
            for x in (xpxl1 + 1.0) as i32..(xpxl2 - 1.0) as i32 {
                plot(self, ipart(intery), x as f32, rfpart(intery), rgba);
                plot(self, ipart(intery) + 1.0, x as f32, fpart(intery), rgba);
                intery += grad;
            }
        } else {
            for x in (xpxl1 + 1.0) as i32..(xpxl2 - 1.0) as i32 {
                plot(self, x as f32, ipart(intery), rfpart(intery), rgba);
                plot(self, x as f32, ipart(intery) + 1.0, fpart(intery), rgba);
                intery += grad;
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

    #[test]
    fn test_line() {
        let mut pic = Picture::new(512, 512);
        pic.line_aa(0, 0, 512, 512, &(1.0, 0.0, 0.0, 1.0).into());
        pic.line_aa(0, 0, 10, 512, &(1.0, 0.0, 0.0, 1.0).into());
        pic.save("test.png");
    }
}
