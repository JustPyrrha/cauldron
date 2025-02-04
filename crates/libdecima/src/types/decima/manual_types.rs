use std::marker::PhantomData;
use std::slice;

// todo: rewrite this and move things to their correct place

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Array<T> {
    pub count: u32,
    pub capacity: u32,
    pub entries: *mut T,
}

impl<T> Array<T> {
    pub fn slice(&self) -> &[T] {
        if self.count == 0 {
            &[]
        } else {
            unsafe { slice::from_raw_parts(self.entries, self.count as usize) }
        }
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Ref<T> {
    pub ptr: *mut T,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct StreamingRef<T> {
    pub ptr: *mut T,
}


#[derive(Debug, Clone)]
#[repr(C)]
pub struct UUIDRef<T> {
    // pub uuid: crate::generated::GGUUID,
    pub uuid: [u8; 16],
    pub marker: PhantomData<T>,
}

// todo: reverse WeakPtr
#[derive(Debug, Clone)]
#[repr(C)]
pub struct WeakPtr<T> {
    pub unk: [u8; 32],
    pub marker: PhantomData<T>,
}

#[allow(non_camel_case_types)]
pub type cptr<T> = Ref<T>;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct HashMapEntry<V> {
    pub value: V,
    pub hash: u32,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct HashMap<T> {
    pub entries: *mut HashMapEntry<T>,
    pub size: u32,
    pub capacity: u32,
}
