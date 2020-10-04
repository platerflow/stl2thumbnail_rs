mod aabb;
mod encoder;
mod ffi;
mod mesh;
mod parser;
mod picture;
mod rasterbackend;
mod zbuffer;

use anyhow::Result;
use encoder::encode_gif;
use encoder::*;
use mesh::{LazyMesh, Mesh};
use mesh::{Triangle, Vec3};
use parser::Parser;
use picture::Picture;
use rasterbackend::RasterBackend;

use clap::{App, Arg};
use std::error::Error;

struct Flags {
    verbose: bool,
    lazy: bool,
    recalculate_normals: bool,
    turntable: bool,
    size_hint: bool,
}

fn main() -> Result<()> {
    let matches = App::new("stl2thumbnail")
        .version(clap::crate_version!())
        .about("Generates thumbnails from STL files")
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
        .arg(Arg::with_name("VERBOSE").short("v").long("verbose").help("Be verbose"))
        .arg(
            Arg::with_name("LAZY")
                .short("l")
                .long("lazy")
                .help("Enables low memory usage mode"),
        )
        .arg(
            Arg::with_name("RECALC_NORMALS")
                .short("n")
                .long("normals")
                .help("Always recalculate normals"),
        )
        .arg(
            Arg::with_name("WIDTH")
                .short("w")
                .long("width")
                .takes_value(true)
                .help("Width of the generated image (defaults to 256)"),
        )
        .arg(
            Arg::with_name("HEIGHT")
                .short("h")
                .long("height")
                .takes_value(true)
                .help("Height of the generated image (defaults to 256)"),
        )
        .arg(
            Arg::with_name("SIZE_HINT")
                .short("d")
                .long("dimensions")
                .help("Draws the dimensions underneath the model (requires height of at least 128 pixels)"),
        )
        .get_matches();

    let input = matches.value_of("INPUT").unwrap();
    let output = matches.value_of("OUTPUT").unwrap();

    let width = matches.value_of("WIDTH").unwrap_or("").parse::<usize>().unwrap_or(256);
    let height = matches.value_of("HEIGHT").unwrap_or("").parse::<usize>().unwrap_or(256);

    let flags = Flags {
        verbose: matches.is_present("VERBOSE"),
        lazy: matches.is_present("LAZY"),
        recalculate_normals: matches.is_present("RECALC_NORMALS"),
        size_hint: matches.is_present("SIZE_HINT") && height >= 128,
        turntable: matches.is_present("TURNTABLE"),
    };

    let mut parser = Parser::from_file(&input, flags.recalculate_normals)?;

    if flags.lazy {
        let parsed_mesh = LazyMesh::new(parser);

        if flags.verbose {
            println!("Size                  '{}x{}'", width, height);
            println!("Input                 '{}'", input);
            println!("Output                '{}'", output);
            println!("Recalculate normals   '{}'", flags.recalculate_normals);
            println!("Low memory usage mode '{}'", flags.lazy);
        }

        create(width, height, &parsed_mesh, 25.0, &output, &flags)?;
    } else {
        let parsed_mesh = parser.read_all()?;

        if flags.verbose {
            println!("Size                  '{}x{}'", width, height);
            println!("Input                 '{}'", input);
            println!("Output                '{}'", output);
            println!("Recalculate normals   '{}'", flags.recalculate_normals);
            println!("Low memory usage mode '{}'", flags.lazy);
        }

        create(width, height, &parsed_mesh, 25.0, &output, &flags)?;
    }

    Ok(())
}

fn create(
    width: usize,
    height: usize,
    mesh: impl IntoIterator<Item = Triangle> + Copy,
    elevation_angle: f32,
    path: &str,
    flags: &Flags,
) -> Result<()> {
    if flags.turntable {
        create_turntable_animation(width, height, mesh, elevation_angle, path, flags)
    } else {
        create_still(width, height, mesh, elevation_angle, path, flags)
    }
}

fn create_still(
    width: usize,
    height: usize,
    mesh: impl IntoIterator<Item = Triangle> + Copy,
    elevation_angle: f32,
    path: &str,
    flags: &Flags,
) -> Result<()> {
    let elevation_angle = elevation_angle * std::f32::consts::PI / 180.0;
    let mut backend = RasterBackend::new(width, height);

    backend.render_options.view_pos = Vec3::new(1.0, 1.0, -elevation_angle.tan());
    let (aabb, scale) = backend.fit_mesh_scale(mesh);
    backend.render_options.zoom = 1.05;
    backend.render_options.draw_size_hint = flags.size_hint;

    backend.render(mesh, scale, &aabb).save(path)?;

    Ok(())
}

fn create_turntable_animation(
    width: usize,
    height: usize,
    mesh: impl IntoIterator<Item = Triangle> + Copy,
    elevation_angle: f32,
    path: &str,
    flags: &Flags,
) -> Result<()> {
    let elevation_angle = elevation_angle * std::f32::consts::PI / 180.0;
    let mut backend = RasterBackend::new(width, height);
    //backend.render_options.grid_visible = false;
    let mut pictures: Vec<Picture> = Vec::new();

    backend.render_options.view_pos = Vec3::new(1.0, 1.0, -elevation_angle.tan());
    let (aabb, scale) = backend.fit_mesh_scale(mesh);
    backend.render_options.zoom = 1.05;
    backend.render_options.draw_size_hint = flags.size_hint;

    for i in 0..45 {
        let angle = 8.0 * i as f32 * std::f32::consts::PI / 180.0;
        backend.render_options.view_pos = Vec3::new(angle.cos(), angle.sin(), -elevation_angle.tan());
        pictures.push(backend.render(mesh, scale, &aabb));
    }

    encode_gif(path, pictures.as_slice())?;

    Ok(())
}
