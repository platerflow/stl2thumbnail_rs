use crate::com_interface::{IInitializeWithStream, ISequentialStream, IStream, IThumbnailProvider};
use com::co_class;
use com::sys::{HRESULT, NOERROR};
use com::ComPtr;

use std::ptr::null_mut;
use winapi::shared::minwindef::{DWORD, PINT, UINT};
use winapi::shared::windef::HBITMAP;
use winapi::shared::wtypes::STATFLAG_NONAME;
use winapi::um::objidlbase::STATSTG;
use winapi::um::wingdi::CreateBitmap;
use winapi::um::wingdi::BITMAP;
use winapi::um::winnt::ULONGLONG;

use std::cell::RefCell;
use std::ffi::c_void;

use std::io::Cursor;
use stl2thumbnail::parser::Parser;
use stl2thumbnail::picture::Picture;
use stl2thumbnail::rasterbackend::RasterBackend;

#[co_class(implements(IThumbnailProvider, IInitializeWithStream))]
pub struct WinSTLThumbnailGenerator {
    data: RefCell<Vec<u8>>,
}

impl WinSTLThumbnailGenerator {
    pub unsafe fn new() -> Box<Self> {
        Self::allocate(
            RefCell::new(Vec::new()),
        )
    }
}

impl IThumbnailProvider for WinSTLThumbnailGenerator {
    unsafe fn get_thumbnail(
        &self,
        cx: UINT,            // size in x & y dimension
        phbmp: *mut HBITMAP, // data ptr
        pdw_alpha: PINT,
    ) -> com::sys::HRESULT {
        *phbmp = null_mut();
        *pdw_alpha = 0x0; // WTSAT_UNKNOWN

        let slice = &self.data.borrow()[..];
        let reader = Cursor::new(slice);
        let mut parser = Parser::from_buf(reader, false).unwrap();

        if let Ok(mesh) = parser.read_all() {
            let mut backend = RasterBackend::new(cx as usize, cx as usize);
            let (aabb, scale) = backend.fit_mesh_scale(&mesh);
            backend.render_options.zoom = 1.05;
            // Note: Icon cache sizes seem to be 16,32,48,96,256,768,1280,...
            // 256 this is actually too small to be readable when it first appears,
            // but 768 makes no sense either
            backend.render_options.draw_size_hint = cx >= 256;
            let pic = backend.render(&mesh, scale, &aabb);

            *phbmp = create_hbitmap_from_picture(&pic);
            *pdw_alpha = 0x2; // WTSAT_ARGB

            return NOERROR;
        }

        -1 // error

        // **this works for testing**
        // let mut pic = Picture::new(cx as usize, cx as usize);
        // pic.test_pattern();
        // *phbmp = create_hbitmap_from_picture(&pic);
        // *pdw_alpha = 0x2; // WTSAT_ARGB

        // return NOERROR
    }
}

impl IInitializeWithStream for WinSTLThumbnailGenerator {
    unsafe fn initialize(&self, pstream: ComPtr<dyn IStream>, grf_mode: DWORD) -> HRESULT {
        if let Some(stream) = pstream.get_interface::<dyn IStream>() {
            // figure out the length of the stream
            let mut stat: STATSTG = std::mem::zeroed();

            if stream.stat(&mut stat, STATFLAG_NONAME) != NOERROR {
                return -2;
            }

            let len = *stat.cbSize.QuadPart() as usize;

            println!("Got stream of length {}", len);

            // read the entire stream
            self.data.replace(vec![0; len as usize]);
            stream.read(
                self.data.borrow_mut().as_mut_ptr() as *mut c_void,
                len as u32,
                std::ptr::null_mut(),
            );

            return NOERROR;
        } else {
            println!("Error: Invalid stream interface");
            return -1;
        }
    }
}

fn create_hbitmap_from_picture(pic: &Picture) -> HBITMAP {
    let bgra_data = pic.to_bgra();
    let data = bgra_data.as_ptr() as *mut winapi::ctypes::c_void;
    let width = pic.width() as i32;
    let height = pic.height() as i32;

    // Windows allocates the memory for this bitmap and copies the 'data' to its own buffer
    // Important: The image format here is B8G8R8A8
    unsafe { CreateBitmap(width, height, 1, 32, data) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn win_bitmap() {
        let mut pic = Picture::new(512, 512);
        pic.test_pattern();

        let hbitmap = create_hbitmap_from_picture(&pic);

        assert!(hbitmap != null_mut());
    }
}
