use std::{ptr, slice};
use libc::{c_uchar};

/*
extern "C" {
    fn allocate_data(data_ptr: *mut *const c_uchar, data_len: *mut i32);
    fn deallocate_data(data_ptr: *const c_uchar);
}
*/

//fn deallocate_data(data_ptr: *const c_uchar);

pub struct CVec {
    ptr: *const c_uchar,
    len: usize,
}

impl CVec{
    pub fn new(ptr: *const c_uchar, len: usize) -> CVec {
        CVec{
            ptr: ptr,
            len: len
        }
    }
}

impl std::ops::Deref for CVec {
    type Target = [c_uchar];

    fn deref(&self) -> &[c_uchar] {
        unsafe { slice::from_raw_parts(self.ptr, self.len) }
    }
}

impl Drop for CVec {
    fn drop(&mut self) {
        //unsafe { deallocate_data(self.ptr) };
    }
}

fn get_vec() -> CVec {
    let mut ptr = ptr::null();
    let mut len = 0;

    unsafe {
        //allocate_data(&mut ptr, &mut len);
        assert!(!ptr.is_null());
        assert!(len >= 0);

        CVec {
            ptr,
            len: len as usize,
        }
    }
}