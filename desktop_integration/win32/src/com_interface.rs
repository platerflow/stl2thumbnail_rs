use com::ComPtr;
use com::{com_interface, interfaces::iunknown::IUnknown, sys::HRESULT};

use winapi::shared::minwindef::{DWORD, PINT, UINT, ULONG};
use winapi::shared::windef::HBITMAP;
use winapi::um::objidlbase::STATSTG;
use winapi::um::winnt::{LONGLONG, ULONGLONG};

use std::ffi::c_void;

#[com_interface("0000000c-0000-0000-C000-000000000046")]
pub trait IStream: ISequentialStream {
    unsafe fn clone(&self, ppstm: ComPtr<dyn IStream>) -> HRESULT;
    unsafe fn commit(&self, commit_flags: DWORD) -> HRESULT;
    unsafe fn copy_to(
        &self,
        ppstm: ComPtr<dyn IStream>,
        cb: ULONGLONG,
        pcb_read: *mut ULONGLONG,
        pcb_written: *mut ULONGLONG,
    ) -> HRESULT;
    unsafe fn lock_region(&self, lib_offset: ULONGLONG, cb: ULONGLONG, lock_type: DWORD) -> HRESULT;
    unsafe fn revert(&self) -> HRESULT;
    unsafe fn seek(&self, dlib_move: LONGLONG, dw_origin: DWORD, plib_new_position: *mut ULONGLONG) -> HRESULT;
    unsafe fn set_size(&self, lib_new_size: ULONGLONG) -> HRESULT;
    unsafe fn stat(&self, pstatstg: *mut STATSTG, grf_stat_flag: DWORD) -> HRESULT;
    unsafe fn unlock_region(&self, lib_offset: ULONGLONG, cb: ULONGLONG, lock_type: DWORD) -> HRESULT;
}

#[com_interface("14AD64D8-F24A-402F-A5EF-FA4E16AA4ED4")]
pub trait ISequentialStream: IUnknown {
    unsafe fn read(&self, pv: *mut c_void, cb: ULONG, pcb_read: *mut ULONG) -> HRESULT;
    unsafe fn write(&self, pv: *mut c_void, cb: ULONG, pcb_written: *mut ULONG) -> HRESULT;
}

#[com_interface("E357FCCD-A995-4576-B01F-234630154E96")]
pub trait IThumbnailProvider: IUnknown {
    unsafe fn get_thumbnail(&self, cx: UINT, phbmp: *mut HBITMAP, pdw_alpha: PINT) -> HRESULT;
}

#[com_interface("b824b49d-22ac-4161-ac8a-9916e8fa3f7f")]
pub trait IInitializeWithStream: IUnknown {
    unsafe fn initialize(&self, pstream: ComPtr<dyn IStream>, grf_mode: DWORD) -> HRESULT;
}
