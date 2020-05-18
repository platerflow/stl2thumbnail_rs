use crate::mesh::*;
use anyhow::*;
use byteorder::{LittleEndian, ReadBytesExt};
use scan_fmt::*;
use std::fs;
use std::io;
use std::io::{BufReader, Read, Seek, BufRead};

const HEADER_SIZE: u64 = 80;
const TRIANGLE_SIZE: u64 = 50;

pub enum StlType {
    Binary,
    Ascii,
}

pub struct Parser<T>
where
    T: Read + Seek,
{
    reader: BufReader<T>,
    stl_type: StlType,
    header_length: u64,
}

impl<T: Read + Seek> Parser<T> {
    pub fn from_buf(inner: T) -> Result<Self> {
        let mut reader = BufReader::new(inner);

        let stl_type = deduce_stl_type(&mut reader)?;
        reader.seek(io::SeekFrom::Start(0))?;

        // figure out header size
        let mut header_length= 0;
        match stl_type {
            StlType::Binary => {
                header_length = HEADER_SIZE;
            },
            StlType::Ascii => {
                while let Some(line) = read_ascii_line(&mut reader).ok() {
                    if line.starts_with("solid") {
                        header_length = reader.seek(io::SeekFrom::Current(0))? as u64;
                        break;
                    }
                }
            }
        };

        Ok(Self { reader, stl_type, header_length })
    }

    pub fn rewind(&mut self) -> Result<()> {
        self.reader.seek(std::io::SeekFrom::Start(self.header_length))?;
        Ok(())
    }

    pub fn next_triangle(&mut self) -> Option<Triangle> {
        match self.stl_type {
            StlType::Ascii => read_ascii_triangle(&mut self.reader).ok(),
            StlType::Binary => read_triangle(&mut self.reader).ok(),
        }
    }

    pub fn triangle_count(&mut self) -> Result<u64> {
        self.rewind();

        match self.stl_type {
            StlType::Binary => {
                // for binary files we can quickly read the first u32
                // after the header which contains the triangle count
                self.rewind();
                let count = self.reader.read_u32::<LittleEndian>()? as u64;
                return Ok(count);
            },
            StlType::Ascii => {
                // we have no other choice as parsing the hole file
                let mut count = 0;
                while let Some(_) = self.next_triangle() {
                    count += 1;
                }
                return Ok(count);
            }
        }
    }

    pub fn read_all(&mut self) -> Result<Mesh> {
        self.rewind();
        let mut mesh = Mesh::new();

        while let Some(triangle) = self.next_triangle() {
            mesh.push(triangle);
        }

        Ok(mesh)
    }
}

impl Parser<fs::File> {
    pub fn from_file(filename: &str) -> Result<Self> {
        let mut file = fs::File::open(filename)?;
        // let mut reader = BufReader::new(file);
        //Self::from_buf(Box::new(file))
        // let stl_type = deduce_stl_type(&mut reader)?;
        // reader.seek(std::io::SeekFrom::Start(0))?;

        Self::from_buf(file)
    }
}

pub struct ParserIterator<'a, T>
where
    T: std::io::Read + std::io::Seek,
{
    parser: &'a mut Parser<T>,
}

impl<'a, T: std::io::Read + std::io::Seek> IntoIterator for &'a mut Parser<T> {
    type Item = Triangle;
    type IntoIter = ParserIterator<'a, T>;

    fn into_iter(mut self) -> Self::IntoIter {
        self.rewind();
        Self::IntoIter { parser: self }
    }
}

impl<T: std::io::Read + std::io::Seek> Iterator for ParserIterator<'_, T> {
    type Item = Triangle;

    fn next(&mut self) -> Option<Self::Item> {
        self.parser.next_triangle()
    }
}

// impl<T: Read> BufReader<T> {
//     fn parse_triangle() -> Option<Triangle> {
//         None
//     }
// }

pub fn parse_file(filename: &str) -> Result<Mesh> {
    let mut file = fs::File::open(filename)?;
    let mut reader = BufReader::new(file);

    parse(&mut reader)
}

pub fn parse<T: io::BufRead + io::Seek>(reader: &mut T) -> Result<Mesh> {
    let file_type = deduce_stl_type(reader)?;
    let mut mesh = Mesh::new();

    match file_type {
        StlType::Binary => {
            // skip header
            reader.seek(std::io::SeekFrom::Start(HEADER_SIZE))?;

            // get the vertex count
            let vertex_count = reader.read_u32::<LittleEndian>()?;

            // reserve memory
            mesh.reserve(vertex_count as usize);

            for _ in 0..vertex_count {
                let triangle = read_triangle(reader)?; // triangle
                reader.read_u16::<LittleEndian>()?; // attributes

                mesh.push(triangle);
            }
        }

        StlType::Ascii => {
            reader.seek(std::io::SeekFrom::Start(0))?;

            read_ascii_line(reader)?; // solid ...

            while let Some(triangle) = read_ascii_triangle(reader).ok() {
                mesh.push(triangle);
            }
        }
    }

    Ok(mesh)
}

fn deduce_stl_type<T: io::BufRead + io::Seek>(reader: &mut T) -> Result<StlType> {
    // skip header
    reader.seek(std::io::SeekFrom::Start(HEADER_SIZE))?;

    // the best way to distinguish between 'ascii' and 'bin' files is to check whether the
    // specified triangle count matches the size of the file
    let triangles = reader.read_u32::<LittleEndian>()? as u64;
    let filesize = reader.seek(std::io::SeekFrom::End(0))?;
    if triangles * TRIANGLE_SIZE + HEADER_SIZE + std::mem::size_of::<u32>() as u64 == filesize {
        return Ok(StlType::Binary);
    }

    // Note: also malformed binary STL files get classified as 'ascii'
    Ok(StlType::Ascii)
}

fn read_ascii_line<T: io::BufRead>(reader: &mut T) -> Result<String> {
    let mut line = String::new();
    reader.read_line(&mut line)?;
    Ok(line.trim_start().to_ascii_lowercase())
}

fn read_ascii_triangle<T: io::BufRead>(reader: &mut T) -> Result<Triangle> {
    let mut vertices = [
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 0.0),
    ];

    let (nx, ny, nz) = scan_fmt!(
        &read_ascii_line(reader)?,
        "facet normal {f} {f} {f}",
        f32,
        f32,
        f32
    )?;

    read_ascii_line(reader)?; // "outer loop"

    for i in 0..3 {
        let (vx, vy, vz) = scan_fmt!(
            &read_ascii_line(reader)?,
            "vertex {f} {f} {f}",
            f32,
            f32,
            f32
        )?;
        vertices[i].x = vx;
        vertices[i].y = vy;
        vertices[i].z = vz;
    }

    read_ascii_line(reader)?; // "endloop"
    read_ascii_line(reader)?; // "endfacet"

    Ok(Triangle::new(vertices, Vec3::new(nx, ny, nz)))
}

fn read_vec3<T: io::Read>(reader: &mut T) -> Result<Vec3> {
    let mut v = [0.0; 3];

    v[0] = reader.read_f32::<LittleEndian>()?;
    v[1] = reader.read_f32::<LittleEndian>()?;
    v[2] = reader.read_f32::<LittleEndian>()?;

    Ok(Vec3::new(v[0], v[1], v[2]))
}

fn read_triangle<T: io::Read>(reader: &mut T) -> Result<Triangle> {
    let mut n = read_vec3(reader)?;
    let v1 = read_vec3(reader)?;
    let v2 = read_vec3(reader)?;
    let v3 = read_vec3(reader)?;

    // calculate normal from vertices using right hand rule is case it is missing
    if n == Vec3::new(0.0, 0.0, 0.0) || n == Vec3::new(std::f32::NAN, std::f32::NAN, std::f32::NAN)
    {
        n = (v2 - v1).cross(&(v3 - v1)).normalize();
    }

    Ok(Triangle::new([v1, v2, v3], n))
}

#[cfg(test)]
mod test {
    use crate::mesh::*;
    use crate::parser::{parse, Parser};
    use std::io::Cursor;

    const TRI_BIN: &'static [u8] = include_bytes!("test_models/triangle.stl");
    const TRI_ASCII: &'static [u8] = include_bytes!("test_models/triangle_ascii.stl");
    const TRI_ASCII_BROKEN: &'static [u8] = include_bytes!("test_models/triangle_ascii_broken.stl");

    #[test]
    fn parser_ascii_test() {
        let mut reader = Cursor::new(TRI_ASCII);

        let mesh = parse(&mut reader).unwrap();

        assert_eq!(mesh[0].normal, Vec3::new(0.0, 0.0, 1.0));
        assert_eq!(mesh[0].vertices[0], Vec3::new(-1.0, -1.0, 0.0));
        assert_eq!(mesh[0].vertices[1], Vec3::new(1.0, -1.0, 0.0));
        assert_eq!(mesh[0].vertices[2], Vec3::new(0.0, 1.0, 0.0));
    }

    #[test]
    fn parser_ascii_broken_test() {
        let mut reader = Cursor::new(TRI_ASCII_BROKEN);

        let mesh = parse(&mut reader).unwrap();
        assert_eq!(mesh.len(), 0);
    }

    #[test]
    fn parser_bin_test() {
        let mut reader = Cursor::new(TRI_BIN);

        let mesh = parse(&mut reader).unwrap();

        assert_eq!(mesh[0].normal, Vec3::new(0.0, 0.0, 1.0));
        assert_eq!(mesh[0].vertices[0], Vec3::new(-1.0, -1.0, 0.0));
        assert_eq!(mesh[0].vertices[1], Vec3::new(1.0, -1.0, 0.0));
        assert_eq!(mesh[0].vertices[2], Vec3::new(0.0, 1.0, 0.0));
    }

    #[test]
    fn parser_bin_test_lazy() {
        let mut reader = Cursor::new(TRI_BIN);
        let mut parser = Parser::from_buf(reader).unwrap();

        let tri = parser.next_triangle();
        println!("{:?}", tri.unwrap());

        //parser.rewind();
        for tri in &mut parser {
            println!("{:?}", tri);
        }

        for tri in &mut parser {
            println!("{:?}", tri);
        }

        some_function(&mut parser);

        // let mesh = parse(&mut reader).unwrap();
        //
        // assert_eq!(mesh[0].normal, Vec3::new(0.0, 0.0, 1.0));
        // assert_eq!(mesh[0].vertices[0], Vec3::new(-1.0, -1.0, 0.0));
        // assert_eq!(mesh[0].vertices[1], Vec3::new(1.0, -1.0, 0.0));
        // assert_eq!(mesh[0].vertices[2], Vec3::new(0.0, 1.0, 0.0));
    }

    #[test]
    fn mesh_lazy_bin() {
        let mut reader = Cursor::new(TRI_BIN);
        let mut parser = Parser::from_buf(reader).unwrap();
        let mut lazy_mesh = LazyMesh::new(&mut parser);

        for tri in &mut lazy_mesh {
            println!("{:?}", tri);
        }

        some_function(&mut parser);

        // let mesh = parse(&mut reader).unwrap();
        //
        // assert_eq!(mesh[0].normal, Vec3::new(0.0, 0.0, 1.0));
        // assert_eq!(mesh[0].vertices[0], Vec3::new(-1.0, -1.0, 0.0));
        // assert_eq!(mesh[0].vertices[1], Vec3::new(1.0, -1.0, 0.0));
        // assert_eq!(mesh[0].vertices[2], Vec3::new(0.0, 1.0, 0.0));
    }

    #[test]
    fn mesh_lazy_ascii() {
        let mut reader = Cursor::new(TRI_ASCII);
        let mut parser = Parser::from_buf(reader).unwrap();
        let mut lazy_mesh = LazyMesh::new(&mut parser);


        // for tri in &mut lazy_mesh {
        //     println!("{:?}", tri);
        // }

        // for tri in &mut lazy_mesh {
        //     println!("{:?}", tri);
        // }

        // {
        //     let v: Vec<_> = (&mut lazy_mesh).into_iter().collect();
        //     println!("{}", v.len());
        // }

        //
        some_function(&mut lazy_mesh);
        some_function(&mut lazy_mesh);

        // let mesh = parse(&mut reader).unwrap();
        //
        // assert_eq!(mesh[0].normal, Vec3::new(0.0, 0.0, 1.0));
        // assert_eq!(mesh[0].vertices[0], Vec3::new(-1.0, -1.0, 0.0));
        // assert_eq!(mesh[0].vertices[1], Vec3::new(1.0, -1.0, 0.0));
        // assert_eq!(mesh[0].vertices[2], Vec3::new(0.0, 1.0, 0.0));
    }

    fn some_function(m: impl IntoIterator<Item = Triangle>) {
        for tri in m {
            println!("{:?}", tri);
        }
    }
}
