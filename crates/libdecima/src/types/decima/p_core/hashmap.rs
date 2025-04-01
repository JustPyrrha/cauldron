use core::fmt::Debug;
use std::slice;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct HashSetEntry<T>
where
    T: Debug + Clone,
{
    pub hash: u32,
    pub value: T,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct HashSet<T>
where
    T: Debug + Clone,
{
    pub entries: *mut HashSetEntry<T>,
    pub count: u32,
    pub capacity: u32,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct HashMapValue<K, V>
where
    K: Debug + Clone,
    V: Debug + Clone,
{
    pub key: K,
    pub value: V,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct HashMapEntry<K, V>
where
    K: Debug + Clone,
    V: Debug + Clone,
{
    pub value: HashMapValue<K, V>,
    pub hash: u32,
}

impl<K: Debug + Clone, V: Debug + Clone> HashMap<K, V> {
    pub fn slice(&self) -> &[HashMapEntry<K, V>] {
        if self.count == 0 {
            &[]
        } else {
            unsafe { slice::from_raw_parts(self.entries, self.count as usize) }
        }
    }

    pub fn slice_mut(&mut self) -> &mut [HashMapEntry<K, V>] {
        if self.count == 0 {
            &mut []
        } else {
            unsafe { slice::from_raw_parts_mut(self.entries, self.count as usize) }
        }
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct HashMap<K, V>
where
    K: Debug + Clone,
    V: Debug + Clone,
{
    pub entries: *mut HashMapEntry<K, V>,
    pub count: u32,
    pub capacity: u32,
}

impl<T: Debug + Clone> HashSet<T> {
    pub fn slice(&self) -> &[HashSetEntry<T>] {
        if self.count == 0 {
            &[]
        } else {
            unsafe { slice::from_raw_parts(self.entries, self.count as usize) }
        }
    }

    pub fn slice_mut(&mut self) -> &mut [HashSetEntry<T>] {
        if self.count == 0 {
            &mut []
        } else {
            unsafe { slice::from_raw_parts_mut(self.entries, self.count as usize) }
        }
    }
}
