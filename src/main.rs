mod aabb;
mod encoder;
mod ffi;
mod mesh;
mod parser;
mod picture;
mod rasterbackend;
mod zbuffer;

use encoder::encode_gif;
use encoder::*;
use mesh::{LazyMesh, Mesh};
use mesh::{Triangle, Vec3};
use parser::Parser;
use picture::Picture;
use rasterbackend::RasterBackend;

use clap::{App, Arg};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("stl2thumbnail")
        .version(clap::crate_version!())
        .about("Generates thumbnails for STL files")
        .arg(
            Arg::with_name("INPUT")
                .short("i")
                .index(1)
                .long("input")
                .help("Input filename")
                .required(true),
        )
        .arg(
            Arg::with_name("OUTPUT")
                .short("o")
                .index(2)
                .long("output")
                .help("Output filename")
                .required(true),
        )
        .arg(
            Arg::with_name("TURNTABLE")
                .short("t")
                .long("turntable")
                .help("Enables turntable mode"),
        )
        .arg(
            Arg::with_name("VERBOSE")
                .short("v")
                .long("verbose")
                .help("Be verbose"),
        )
        .arg(
            Arg::with_name("LAZY")
                .short("l")
                .long("lazy")
                .help("Be lazy (low memory mode)"),
        )
        .arg(
            Arg::with_name("RECALC_NORMALS")
                .short("n")
                .long("normals")
                .help("Always recalculate normals"),
        )
        .get_matches();

    let input = matches.value_of("INPUT").unwrap();
    let output = matches.value_of("OUTPUT").unwrap();
    let verbose = matches.occurrences_of("VERBOSE") > 0;
    let lazy = matches.occurrences_of("LAZY") > 0;
    let recalculate_normals = matches.occurrences_of("RECALC_NORMALS") > 0;

    let mut parser = Parser::from_file(&input, recalculate_normals)?;

    if lazy {
        let parsed_mesh = LazyMesh::new(parser); //parser::parse_file(&input)?;

        if verbose {
            println!("Input     \"{}\"", input);
            println!("Output    \"{}\"", output);
            println!("Lazy");
            // println!("Triangles {}", parsed_mesh.len());
            // println!("Vertices  {}", parsed_mesh.len() * 3);
        }

        if matches.occurrences_of("TURNTABLE") > 0 {
            create_turntable_animation(&parsed_mesh, 25.0, &output)?;
        } else {
            create_still(&parsed_mesh, 25.0, &output)?;
        }
    } else {
        let parsed_mesh = parser.read_all().unwrap();

        if verbose {
            println!("Input     \"{}\"", input);
            println!("Output    \"{}\"", output);
            println!("Greedy");
            // println!("Triangles {}", parsed_mesh.len());
            // println!("Vertices  {}", parsed_mesh.len() * 3);
        }

        if matches.occurrences_of("TURNTABLE") > 0 {
            create_turntable_animation(&parsed_mesh, 25.0, &output)?;
        } else {
            create_still(&parsed_mesh, 25.0, &output)?;
        }
    }

    Ok(())
}

fn create_still(
    mesh: impl IntoIterator<Item = Triangle> + Copy,
    elevation_angle: f32,
    path: &str,
) -> Result<(), std::io::Error> {
    let elevation_angle = elevation_angle * std::f32::consts::PI / 180.0;
    let mut backend = RasterBackend::new(256, 256);

    backend.view_pos = Vec3::new(1.0, 1.0, -elevation_angle.tan());
    let scale = backend.fit_mesh_scale(mesh);
    backend.zoom = 1.05;

    backend.render(mesh, scale).save(path)?;

    Ok(())
}

fn create_turntable_animation(
    mesh: impl IntoIterator<Item = Triangle> + Copy,
    elevation_angle: f32,
    path: &str,
) -> Result<(), std::io::Error> {
    let elevation_angle = elevation_angle * std::f32::consts::PI / 180.0;
    let mut backend = RasterBackend::new(256, 256);
    backend.grid_visible = false;
    let mut pictures: Vec<Picture> = Vec::new();

    backend.view_pos = Vec3::new(1.0, 1.0, -elevation_angle.tan());
    let scale = backend.fit_mesh_scale(mesh);
    backend.zoom = 1.05;

    for i in 0..45 {
        let angle = 8.0 * i as f32 * std::f32::consts::PI / 180.0;
        backend.view_pos = Vec3::new(angle.cos(), angle.sin(), -elevation_angle.tan());
        pictures.push(backend.render(mesh, scale));
    }

    encode_gif(path, pictures.as_slice())?;

    Ok(())
}
