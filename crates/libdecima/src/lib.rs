#![feature(once_cell_get_mut)]
// #![feature(macro_metavar_expr_concat)]
#![allow(static_mut_refs)]

#[doc(include = "../README.md")]
#[cfg(not(any(feature = "hfw", feature = "nixxes")))]
compile_error!("At least one target feature must be enabled.");

pub mod mem;
pub mod types;

pub mod macros {
    #[macro_export]
    macro_rules! assert_size {
        ($t:ty, $n:literal) => {
            const _: () = [(); 1][(::core::mem::size_of::<$t>() == $n) as usize ^ 1];
        };
    }

    #[macro_export]
    macro_rules! assert_offset {
        ($t:ty, $f:expr, $n:literal) => {
            const _: () = [(); 1][(::std::mem::offset_of!($t, $f) == $n) as usize ^ 1];
        };
    }

    #[macro_export]
    macro_rules! gen_with_vtbl {
        (
            $name:ident,
            $name_vtbl:ident,
            $(
                fn $func:ident($($arg:ident: $arg_t:ty),*) $(-> $func_ret:ty)?
            );*;
            $(
                pub $field:ident: $field_t:ty
            ),*,
        ) => {
            #[repr(C)]
            #[derive(Debug)]
            #[allow(non_camel_case_types, non_snake_case)]
            pub struct /* VFT */ /*$ {concat($name, _vtbl)}*/ $name_vtbl {
                $(
                    pub $func: extern "C" fn(this: *mut $name $(, $arg: $arg_t)*) $(-> $func_ret)?
                ),*
            }

            #[repr(C)]
            #[derive(Debug)]
            pub struct $name {
                pub __vftable: *mut /*$ {concat($name, _vtbl)}*/ $name_vtbl,
                $(
                    pub $field: $field_t
                ),*
            }

            impl $name {
                pub fn __vftable<'a>(this: *mut $name) -> &'a /*$ {concat($name, _vtbl)}*/ $name_vtbl {
                    let instance = unsafe { &*this };
                    let vftable = unsafe { &*instance.__vftable };
                    vftable
                }

                $(
                    #[allow(non_snake_case)]
                    pub fn $func(this: *mut $name $(, $arg: $arg_t)*) $(-> $func_ret)? {
                        let vftable = Self::__vftable(this as *const _ as *mut _);
                        (vftable.$func)(this $(, $arg)*)
                    }
                )*
            }
        };
    }

    #[macro_export]
    macro_rules! impl_instance {
        ($name:ident, $signature:literal, $instruction_length:literal) => {
            impl $name {
                pub fn get_instance() -> Option<&'static $name> {
                    let ptr = crate::mem::offsets::Offset::from_signature($signature)
                        .unwrap()
                        .as_relative($instruction_length)
                        .as_ptr::<*mut $name>();
                    if !ptr.is_null() {
                        let ptr = unsafe { *ptr };
                        if !ptr.is_null() {
                            let instance = unsafe { &*ptr };
                            return Some(instance);
                        }
                    }
                    None
                }
            }
        };
        ($name:ident, $signature:literal) => {
            impl_instance!($name, $signature, 7);
        };
    }
}

#[cfg(feature = "nixxes")]
pub mod log {
    use crate::types::nixxes::log::NxLogImpl;

    pub fn log_impl(category: &str, text: &str) {
        let log = NxLogImpl::get_instance().unwrap();
        NxLogImpl::fn_log(
            log as *const _ as *mut _,
            format!("{}\0", category).as_str().as_ptr() as *const _,
            format!("{}\0", text).as_str().as_ptr() as *const _,
        )
    }

    #[macro_export]
    macro_rules! log {
        // log!("category", *format! args*);
        ($category:literal, $($arg:tt)*) => {
            $crate::log::log_impl($category, format!($($arg)*).as_str());
        };

        // log!(*format! args*);
        ($($arg:tt)*) => {
            $crate::log::log_impl(module_path!(), format!($($arg)*).as_str());
        };
    }
}

#[cfg(not(feature = "nixxes"))]
pub mod log {
    #[macro_export]
    macro_rules! log {
        // log!("category", *format! args*);
        ($category:literal, $($arg:tt)*) => {
            // todo: unimplemented
        };

        // log!(*format! args*);
        ($($arg:tt)*) => {
            // todo: unimplemented
        };
    }
}
