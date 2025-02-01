use crate::assert_size;
use std::any::Any;
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
// assert_size!(Array<dyn Any>, 16);

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
// assert_size!(Ref<dyn Any>, 8);

#[derive(Debug, Clone)]
#[repr(C)]
pub struct StreamingRef<T> {
    pub ptr: *mut T,
}
// assert_size!(StreamingRef<dyn Any>, 8);

#[derive(Debug, Clone)]
#[repr(C)]
pub struct UUIDRef<T> {
    // pub uuid: crate::generated::GGUUID,
    pub uuid: [u8; 16],
    pub marker: PhantomData<T>,
}
// assert_size!(UUIDRef<dyn Any>, 16);

// todo: reverse WeakPtr
#[derive(Debug, Clone)]
#[repr(C)]
pub struct WeakPtr<T> {
    pub unk: [u8; 32],
    pub marker: PhantomData<T>,
}
// assert_size!(WeakPtr<dyn Any>, 32);

#[allow(non_camel_case_types)]
pub type cptr<T> = Ref<T>;
// assert_size!(cptr<dyn Any>, 8);

#[derive(Debug, Clone)]
#[repr(C)]
pub struct HashMapEntry<V> {
    pub value: V,
    pub hash: u32,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct HashMap<K, V> {
    pub entries: *mut HashMapEntry<V>,
    pub size: u32,
    pub capacity: u32,

    pub key_marker: PhantomData<K>,
}
