use crate::mesh::*;
use anyhow::*;
use byteorder::{LittleEndian, ReadBytesExt};
use scan_fmt::*;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::io::{BufRead, BufReader};

pub fn parse_file(filename: &str) -> Result<Mesh> {
    let mut mesh = Mesh::new();

    let mut file = fs::File::open(filename)?;
    let mut reader = BufReader::new(file);
    let bin = is_binary(&mut reader)?;

    if bin {
        // skip header
        reader.seek(std::io::SeekFrom::Start(80))?;

        // get the vertex count
        let vertex_count = reader.read_u32::<LittleEndian>()?;

        // reserve memory
        mesh.reserve(vertex_count as usize);

        for _ in 0..vertex_count {
            let triangle = read_triangle(&mut reader)?; // triangle
            reader.read_u16::<LittleEndian>()?; // attributes

            mesh.push(triangle);
        }
    } else {
        reader.seek(std::io::SeekFrom::Start(0))?;

        read_ascii_line(&mut reader)?; // solid ...

        while let Some(triangle) = read_ascii_triangle(&mut reader).ok() {
            mesh.push(triangle);
        }
    }

    Ok(mesh)
}

fn is_binary<T: io::Read + io::Seek>(reader: &mut T) -> Result<bool> {
    // we check if the file size matches the number of triangles
    reader.seek(std::io::SeekFrom::Start(80))?;
    let triangles = reader.read_u32::<LittleEndian>()?;
    let filesize = reader.seek(std::io::SeekFrom::End(0))?;

    return Ok((triangles * 50 + 80 + 4) as u64 == filesize)
}

fn read_ascii_line<T: io::BufRead>(reader: &mut T) -> Result<String> {
    let mut line = String::new();
    reader.read_line(&mut line)?;
    Ok(line.trim_start().to_ascii_lowercase())
}

fn read_ascii_triangle<T: io::BufRead>(reader: &mut T) -> Result<Triangle> {
    let mut line = String::new();
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

    let mut t: Triangle = Triangle::new(vertices, Vec3::new(nx, ny, nz));

    Ok(t)
}

fn read_vec3<T: io::Read>(reader: &mut T) -> Result<Vec3> {
    let mut v = [0.0; 3];

    v[0] = reader.read_f32::<LittleEndian>()?; //read_f32(file)?;
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

#[test]
fn parser_test() {
    assert_eq!(true, true)
}
