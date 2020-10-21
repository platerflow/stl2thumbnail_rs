use std::convert::From;
use std::i32;

use crate::mesh::{Vec2, Vec4};
use std::ops::{Add, Mul};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct RGBA {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl RGBA {
    pub fn alpha(&self, a: f32) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a: (self.a as f32 * a) as u8,
        }
    }

    pub fn over(&self, b: RGBA) -> Self {
        // draw self over b
        // Porter-Duff algorithm
        let alpha_a = self.a as f32 / 255.0;
        let alpha_b = b.a as f32 / 255.0;
        let alpha_c = alpha_a + (1.0 - alpha_a) * alpha_b;

        let mut new_p = self * (alpha_a / alpha_c) + b * (((1.0 - alpha_a) * alpha_b) / alpha_c);
        new_p.a = (alpha_c * 255.0) as u8;
        new_p
    }
}

impl Mul<f32> for RGBA {
    type Output = RGBA;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output {
            r: (self.r as f32 * rhs) as u8,
            g: (self.g as f32 * rhs) as u8,
            b: (self.b as f32 * rhs) as u8,
            a: self.a,
        }
    }
}

impl Mul<f32> for &RGBA {
    type Output = RGBA;

    fn mul(self, rhs: f32) -> Self::Output {
        *self * rhs
    }
}

impl Add for RGBA {
    type Output = RGBA;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            r: (self.r as i32 + rhs.r as i32) as u8,
            g: (self.g as i32 + rhs.g as i32) as u8,
            b: (self.b as i32 + rhs.b as i32) as u8,
            a: (self.a as i32 + rhs.a as i32) as u8,
        }
    }
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
        assert_eq!(rgba.len(), 8, "expected format: 'RRGGBBAA'");

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
    width: u32,
    height: u32,
    depth: u32,
}

impl Picture {
    pub fn new(width: u32, height: u32) -> Self {
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

    pub fn stride(&self) -> u32 {
        self.width * self.depth
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn depth(&self) -> u32 {
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
            self.set(x as u32, y as u32, rgba);
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

    pub fn thick_line(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, rgba: &RGBA, width: f32) {
        // Anti-aliased thick line
        // Ref: http://members.chello.at/~easyfilter/bresenham.html
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;
        let ed = if dx + dy == 0 {
            1.0
        } else {
            ((dx * dx + dy * dy) as f32).sqrt()
        };
        let mut x2;
        let mut y2;
        let mut e2;

        let wd = (width + 1.0) / 2.0;
        loop {
            let a = 1.0 - ((err - dx + dy).abs() as f32 / ed - wd).max(0.0);
            self.alpha_blend(x0 as u32, y0 as u32, rgba.alpha(a));
            e2 = err;
            x2 = x0;
            if 2 * e2 >= -dx {
                e2 += dy;
                y2 = y0;
                while (e2 as f32) < ed * wd && (y1 != y2 || dx > dy) {
                    e2 += dx;
                    y2 += sy;
                    let a = 1.0 - (e2.abs() as f32 / ed - wd).max(0.0);
                    self.alpha_blend(x0 as u32, y2 as u32, rgba.alpha(a));
                }

                if x0 == x1 {
                    break;
                }

                e2 = err;
                err -= dy;
                x0 += sx;
            }

            if 2 * e2 <= dy {
                e2 = dx - e2;
                while (e2 as f32) < ed * wd && (x1 != x2 || dx < dy) {
                    e2 += dy;
                    x2 += sx;
                    let a = 1.0 - (e2.abs() as f32 / ed - wd).max(0.0);
                    self.alpha_blend(x2 as u32, y0 as u32, rgba.alpha(a));
                }

                if y0 == y1 {
                    break;
                }

                err += dx;
                y0 += sy;
            }
        }
    }

    pub fn set(&mut self, x: u32, y: u32, rgba: &RGBA) {
        if x >= self.width || y >= self.height {
            return;
        }

        let stride = self.stride();
        self.data[(stride * y + (x * self.depth) + 0) as usize] = rgba.r;
        self.data[(stride * y + (x * self.depth) + 1) as usize] = rgba.g;
        self.data[(stride * y + (x * self.depth) + 2) as usize] = rgba.b;
        self.data[(stride * y + (x * self.depth) + 3) as usize] = rgba.a;
    }

    pub fn alpha_blend(&mut self, x: u32, y: u32, rgba: RGBA) {
        if x >= self.width || y >= self.height {
            return;
        }

        // draw a over b
        let b = self.get(x, y);
        let a = rgba;

        self.set(x, y, &a.over(b));
    }

    pub fn get(&self, x: u32, y: u32) -> RGBA {
        let stride = self.stride();
        RGBA {
            r: self.data[(stride * y + (x * self.depth) + 0) as usize],
            g: self.data[(stride * y + (x * self.depth) + 1) as usize],
            b: self.data[(stride * y + (x * self.depth) + 2) as usize],
            a: self.data[(stride * y + (x * self.depth) + 3) as usize],
        }
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

    pub fn stroke_string(&mut self, x: u32, y: u32, s: &str, char_size: f32, rgba: &RGBA) {
        let mut i = 0;
        for c in s.chars().into_iter() {
            self.stroke_letter(x + i * (char_size * 0.7 + 6.0) as u32, y, c, char_size, rgba);
            i += 1;
        }
    }

    pub fn stroke_letter(&mut self, x: u32, y: u32, c: char, char_size: f32, rgba: &RGBA) {
        let points = match c {
            '0' => vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(0.0, 1.0),
                Vec2::new(0.0, 1.0),
                Vec2::new(0.0, 0.0),
            ],

            '1' => vec![Vec2::new(1.0, 0.0), Vec2::new(1.0, 1.0)],

            '2' => vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 0.5),
                Vec2::new(1.0, 0.5),
                Vec2::new(0.0, 0.5),
                Vec2::new(0.0, 0.5),
                Vec2::new(0.0, 1.0),
                Vec2::new(0.0, 1.0),
                Vec2::new(1.0, 1.0),
            ],

            '3' => vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(0.0, 1.0),
                Vec2::new(1.0, 0.5),
                Vec2::new(0.0, 0.5),
            ],

            '4' => vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(0.0, 0.5),
                Vec2::new(0.0, 0.5),
                Vec2::new(1.0, 0.5),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 1.0),
            ],

            '5' => vec![
                Vec2::new(0.0, 1.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(1.0, 0.5),
                Vec2::new(1.0, 0.5),
                Vec2::new(0.0, 0.5),
                Vec2::new(0.0, 0.5),
                Vec2::new(0.0, 0.0),
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
            ],

            '6' => vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(0.0, 0.0),
                Vec2::new(0.0, 1.0),
                Vec2::new(0.0, 1.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(1.0, 0.5),
                Vec2::new(1.0, 0.5),
                Vec2::new(0.0, 0.5),
            ],

            '7' => vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 1.0),
            ],

            '8' => vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(0.0, 1.0),
                Vec2::new(0.0, 1.0),
                Vec2::new(0.0, 0.0),
                Vec2::new(0.0, 0.5),
                Vec2::new(1.0, 0.5),
            ],

            '9' => vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(0.0, 1.0),
                Vec2::new(0.0, 0.0),
                Vec2::new(0.0, 0.5),
                Vec2::new(0.0, 0.5),
                Vec2::new(1.0, 0.5),
            ],

            'x' => vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(0.0, 1.0),
            ],

            'm' => vec![
                Vec2::new(0.0, 0.5),
                Vec2::new(1.0, 0.5),
                Vec2::new(0.0, 0.5),
                Vec2::new(0.0, 1.0),
                Vec2::new(0.5, 0.5),
                Vec2::new(0.5, 1.0),
                Vec2::new(1.0, 0.5),
                Vec2::new(1.0, 1.0),
            ],

            _ => vec![],
        };

        for p in points.chunks(2) {
            let x0 = p[0].x * char_size * 0.7 + x as f32;
            let y0 = p[0].y * char_size + y as f32;
            let x1 = p[1].x * char_size * 0.7 + x as f32;
            let y1 = p[1].y * char_size + y as f32;
            self.thick_line(x0 as i32, y0 as i32, x1 as i32, y1 as i32, rgba, 3.0);
        }
    }

    pub fn fill_rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, rgba: &RGBA) {
        for x in x0.max(0)..=x1.min(self.width as i32 - 1) {
            for y in y0.max(0)..=y1.min(self.height as i32 - 1) {
                self.set(x as u32, y as u32, rgba);
            }
        }
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
        pic.fill(&(1.0, 1.0, 1.0, 1.0).into());

        pic.thick_line(0, 0, 512, 512, &(1.0, 0.0, 0.0, 1.0).into(), 4.0);
        pic.thick_line(0, 0, 256, 512, &(1.0, 0.0, 0.0, 1.0).into(), 4.0);
        pic.thick_line(0, 256, 512, 256, &(1.0, 0.0, 0.0, 1.0).into(), 4.0);
        pic.thick_line(512, 0, 0, 512, &(1.0, 0.0, 0.0, 1.0).into(), 1.0);

        pic.thick_line(0, 256, 512, 256, &(1.0, 0.0, 0.0, 1.0).into(), 1.0);
        pic.thick_line(256, 0, 256, 512, &(1.0, 0.0, 0.0, 1.0).into(), 1.0);

        // plot chars
        let mut i = 0;
        for c in ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'x', 'm'].iter() {
            pic.stroke_letter(100 + i * 14, 100, *c, 10.0, &"000000FF".into());
            i += 1;
        }

        pic.stroke_string(100, 200, "12x55mm", 10.0, &"E6E6E6FF".into());

        pic.save("test.png").unwrap();
    }
}
