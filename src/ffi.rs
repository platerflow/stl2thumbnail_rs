use std::ffi::CStr;
use std::mem::forget;
use std::os::raw::c_char;

use crate::parser::Parser;
use crate::rasterbackend::RasterBackend;

#[repr(C)]
pub struct PictureBuffer {
    /// data in r8g8b8a8 format
    data: *const u8,
    /// length of the buffer
    len: usize,
}

#[no_mangle]
/// Renders a mesh to a picture
/// Free the buffer with free_picture_buffer
pub extern "C" fn render(path: *const c_char, width: usize, height: usize) -> PictureBuffer {
    let path = unsafe { CStr::from_ptr(path).to_str().unwrap() };

    let backend = RasterBackend::new(width, height);
    let parser = Parser::from_file(path, true);

    if let Ok(mut parser) = parser {
        let mesh = parser.read_all();

        if let Ok(mesh) = mesh {
            let scale = backend.fit_mesh_scale(&mesh);
            let mut pic = backend.render(&mesh, scale);

            let boxed_data = pic.data_as_boxed_slice();
            let data = boxed_data.as_ptr();
            let len = pic.data().len();

            // leak the memory owned by boxed_data
            forget(boxed_data);

            return PictureBuffer { data, len };
        }
    }

    PictureBuffer {
        data: std::ptr::null(),
        len: 0,
    }
}

#[no_mangle]
pub extern "C" fn free_picture_buffer(buffer: PictureBuffer) {
    unsafe {
        let s = std::slice::from_raw_parts_mut(buffer.data as *mut u8, buffer.len);

        // put the memory back into the box such that is can be freed
        Box::from_raw(s as *mut [u8]);
    }
}
