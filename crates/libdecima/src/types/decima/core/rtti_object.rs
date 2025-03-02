#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use crate::types::decima::core::rtti::RTTI;

#[derive(Debug)]
#[repr(C)]
pub struct RTTIObject_vtbl {
    pub GetRTTI: *const extern "C" fn(this: *mut RTTIObject) -> *const RTTI,
    pub Destructor: *mut extern "C" fn(this: *mut RTTIObject),
}

#[derive(Debug)]
#[repr(C)]
pub struct RTTIObject {
    pub __vftable: *const RTTIObject_vtbl,
}

impl RTTIObject {
    pub fn GetRTTI(&mut self) -> &RTTI {
        unsafe { &*(&*(&*self.__vftable).GetRTTI)(self as *mut Self) }
    }
}

impl Drop for RTTIObject {
    fn drop(&mut self) {
        unsafe { (&*(&*self.__vftable).Destructor)(self as *mut Self) }
    }
}
