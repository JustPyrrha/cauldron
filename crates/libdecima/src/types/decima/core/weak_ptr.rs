#![allow(non_camel_case_types)]

use std::marker::PhantomData;

use crate::{assert_size, types::decima::core::rtti::RTTI};

#[repr(C)]
#[derive(Debug)]
pub struct WeakPtrBase {
    pub ptr: *mut WeakPtrTarget,
    pub prev: *mut WeakPtrBase,
    pub next: *mut WeakPtrBase,
}
assert_size!(WeakPtrBase, 0x18);

#[repr(C)]
#[derive(Debug)]
pub struct WeakPtrTarget_vtbl {
    pub fn_dtor: extern "C" fn(),
}

#[repr(C)]
#[derive(Debug)]
pub struct WeakPtrTarget {
    pub __vftable: *mut WeakPtrTarget_vtbl,
    pub head: *mut WeakPtrBase,
}

assert_size!(WeakPtrTarget, 0x10);

#[repr(C)]
#[derive(Debug)]
pub struct WeakPtrRTTITarget_vtbl {
    pub fn_dtor: extern "C" fn(),
    pub fn_get_rtti: extern "C" fn(this: *mut WeakPtrRTTITarget) -> *const RTTI,
}

#[repr(C)]
#[derive(Debug)]
pub struct WeakPtrRTTITarget {
    pub __vftable: *mut WeakPtrRTTITarget_vtbl,
    pub base__: WeakPtrTarget,
}

#[repr(C)]
#[derive(Debug)]
pub struct WeakPtr<T> {
    pub base__: WeakPtrBase,
    pub marker: PhantomData<T>,
}

impl<T> WeakPtr<T> {
    pub fn get(&self) -> *mut T {
        self.base__.ptr as *mut T
    }
}
