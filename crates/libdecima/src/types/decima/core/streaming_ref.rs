use std::{ffi::c_void, marker::PhantomData};

#[repr(C)]
#[derive(Debug)]
pub struct StreamingRefBase {
    pub proxy: *mut c_void, // StreamingRefProxy *
}

#[repr(C)]
#[derive(Debug)]
pub struct StreamingRef<T> {
    pub base: StreamingRefBase,
    pub marker: PhantomData<T>,
}
