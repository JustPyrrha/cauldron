#[derive(Debug, Clone)]
#[repr(C)]
pub struct Ref<T> {
    pub ptr: *mut T,
}
