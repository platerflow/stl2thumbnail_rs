use std::ffi::CStr;
use std::mem::forget;
use std::os::raw::c_char;

use crate::parser::Parser;
use crate::rasterbackend::RasterBackend;
use std::time::Duration;

#[repr(C)]
pub struct PictureBuffer {
    /// data in rgba8888 format
    data: *const u8,
    /// length of the buffer
    len: u32,
    /// stride of the buffer
    stride: u32,
    /// depth of the buffer
    depth: u32,
}

#[repr(C)]
pub struct RenderSettings {
    /// width of the image
    width: u32,
    /// height of the image
    height: u32,
    /// embed a size hint
    size_hint: bool,
    /// max duration of the rendering, 0 to disable
    timeout: u64,
}

#[no_mangle]
/// Renders a mesh to a picture
/// Free the buffer with free_picture_buffer
pub extern "C" fn render(path: *const c_char, settings: RenderSettings) -> PictureBuffer {
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
            let len = pic.data().len() as u32;
            let stride = pic.stride() as u32;
            let depth = pic.depth() as u32;

            // leak the memory owned by boxed_data
            forget(boxed_data);

            return PictureBuffer {
                data,
                len,
                stride,
                depth,
            };
        }
    }

    PictureBuffer {
        data: std::ptr::null(),
        len: 0,
        stride: 0,
        depth: 0,
    }
}

#[no_mangle]
/// Frees the memory of a PictureBuffer
pub extern "C" fn free_picture_buffer(buffer: PictureBuffer) {
    unsafe {
        let s = std::slice::from_raw_parts_mut(buffer.data as *mut u8, buffer.len as usize);

        // put the memory back into the box such that is can be freed
        Box::from_raw(s as *mut [u8]);
    }
}
