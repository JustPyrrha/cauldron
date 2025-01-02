use std::slice;

#[repr(C)]
#[derive(Debug)]
pub struct GGArray<T> {
    pub count: u32,
    pub capacity: u32,
    pub entries: *mut T,
}

impl<T> GGArray<T> {
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn slice(&self) -> &[T] {
        if self.is_empty() {
            &[]
        } else {
            unsafe { slice::from_raw_parts(self.entries, self.count as usize) }
        }
    }
}
