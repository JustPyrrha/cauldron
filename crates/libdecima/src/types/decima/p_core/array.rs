use std::slice;

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Array<T> {
    pub count: u32,
    pub capacity: u32,
    pub data: *mut T,
}

impl<T> Default for Array<T> {
    fn default() -> Self {
        Array {
            count: 0,
            capacity: 0,
            data: std::ptr::null_mut(),
        }
    }
}

impl<T> Array<T> {
    pub fn as_slice(&self) -> &[T] {
        if self.count == 0 {
            &[]
        } else {
            unsafe { slice::from_raw_parts(self.data, self.count as usize) }
        }
    }
}
