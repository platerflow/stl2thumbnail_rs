use std::ffi::CStr;
use std::mem::forget;
use std::os::raw::c_char;

use crate::parser::Parser;
use crate::rasterbackend::RasterBackend;
use std::time::Duration;

#[repr(C)]
pub struct s2t_PictureBuffer {
    /// data in rgba8888 format
    data: *const u8,
    /// length of the buffer
    len: usize,
    stride: usize,
    depth: usize,
}

#[repr(C)]
pub struct s2t_RenderSettings {
    width: usize,
    height: usize,
    size_hint: bool,
    timeout: u64,
}

#[repr(C)]
pub struct s2t_Flags {
    size_hint: bool,
}

#[no_mangle]
/// Renders a mesh to a picture
/// Free the buffer with free_picture_buffer
pub extern "C" fn s2t_render(path: *const c_char, settings: s2t_RenderSettings) -> s2t_PictureBuffer {
    let path = unsafe { CStr::from_ptr(path).to_str().unwrap() };

    let mut backend = RasterBackend::new(settings.width, settings.height);
    let parser = Parser::from_file(path, true);

    if let Ok(mut parser) = parser {
        let mesh = parser.read_all();

        if let Ok(mesh) = mesh {
            let (aabb, scale) = backend.fit_mesh_scale(&mesh);

            // set flags
            backend.render_options.draw_size_hint = settings.size_hint;

            // render
            let mut pic = backend.render(&mesh, scale, &aabb, None);

            let boxed_data = pic.data_as_boxed_slice();
            let data = boxed_data.as_ptr();
            let len = pic.data().len();
            let stride = pic.stride();
            let depth = pic.depth();

            // leak the memory owned by boxed_data
            forget(boxed_data);

            return s2t_PictureBuffer {
                data,
                len,
                stride,
                depth,
            };
        }
    }

    s2t_PictureBuffer {
        data: std::ptr::null(),
        len: 0,
        stride: 0,
        depth: 0,
    }
}

#[no_mangle]
pub extern "C" fn s2t_free_picture_buffer(buffer: s2t_PictureBuffer) {
    unsafe {
        let s = std::slice::from_raw_parts_mut(buffer.data as *mut u8, buffer.len);

        // put the memory back into the box such that is can be freed
        Box::from_raw(s as *mut [u8]);
    }
}
