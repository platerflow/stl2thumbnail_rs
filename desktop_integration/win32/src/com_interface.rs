use com::{interfaces::iunknown::IUnknown, sys::HRESULT};

use winapi::shared::minwindef::{DWORD, PINT, UINT, ULONG};
use winapi::shared::windef::HBITMAP;
use winapi::um::objidlbase::STATSTG;
use winapi::um::winnt::{LONGLONG, ULONGLONG};

use std::ffi::c_void;

com::interfaces! {
    #[uuid("0000000c-0000-0000-C000-000000000046")]
    pub unsafe interface IStream: ISequentialStream {
        pub fn clone(&self, ppstm: IStream) -> HRESULT;
        pub fn commit(&self, commit_flags: DWORD) -> HRESULT;
        pub fn copy_to(
            &self,
            ppstm: IStream,
            cb: ULONGLONG,
            pcb_read: *mut ULONGLONG,
            pcb_written: *mut ULONGLONG,
        ) -> HRESULT;
        pub fn lock_region(&self, lib_offset: ULONGLONG, cb: ULONGLONG, lock_type: DWORD) -> HRESULT;
        pub fn revert(&self) -> HRESULT;
        pub fn seek(&self, dlib_move: LONGLONG, dw_origin: DWORD, plib_new_position: *mut ULONGLONG) -> HRESULT;
        pub fn set_size(&self, lib_new_size: ULONGLONG) -> HRESULT;
        pub fn stat(&self, pstatstg: *mut STATSTG, grf_stat_flag: DWORD) -> HRESULT;
        pub fn unlock_region(&self, lib_offset: ULONGLONG, cb: ULONGLONG, lock_type: DWORD) -> HRESULT;
    }

    #[uuid("14AD64D8-F24A-402F-A5EF-FA4E16AA4ED4")]
    pub unsafe interface ISequentialStream: IUnknown {
        pub fn read(&self, pv: *mut c_void, cb: ULONG, pcb_read: *mut ULONG) -> HRESULT;
        pub fn write(&self, pv: *mut c_void, cb: ULONG, pcb_written: *mut ULONG) -> HRESULT;
    }

    #[uuid("E357FCCD-A995-4576-B01F-234630154E96")]
    pub unsafe interface IThumbnailProvider: IUnknown {
        pub fn get_thumbnail(&self, cx: UINT, phbmp: *mut HBITMAP, pdw_alpha: PINT) -> HRESULT;
    }

    #[uuid("b824b49d-22ac-4161-ac8a-9916e8fa3f7f")]
    pub unsafe interface IInitializeWithStream: IUnknown {
        pub fn initialize(&self, pstream: IStream, grf_mode: DWORD) -> HRESULT;
    }
}
