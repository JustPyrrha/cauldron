#![feature(c_variadic)]
#![feature(once_cell_get_mut)]
#![feature(macro_metavar_expr_concat)]
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
    macro_rules! with_vftable {
        ($name:ident, $(fn $func:ident($($var:ident: $var_ty:ty),*)$(-> $func_r:ty)?),*, $(pub $field:ident: $field_ty:ty),*,) => {
            #[derive(Debug)]
            #[repr(C)]
            pub struct ${concat($name, Vtbl)} {
                $(
                    pub $func: extern "C" fn($($var: $var_ty,)*)$( -> $func_r)?,
                )*
            }

            #[derive(Debug)]
            #[repr(C)]
            pub struct $name {
                pub vtbl: *const ${concat($name, Vtbl)},
                $(
                    pub $field: $field_ty,
                )*
            }
        };
    }
}

#[cfg(feature = "nixxes")]
pub mod log {
    use crate::mem::offsets::Offsets;
    use crate::types::nixxes::log::NxLogImpl;
    use std::slice;

    pub fn log(category: &str, text: &str) {
        Offsets::setup();
        let log_ptr = unsafe {
            *Offsets::resolve_t::<*mut NxLogImpl>("nx::NxLogImpl::Instance")
                .unwrap()
        };
        let instance = unsafe { &*log_ptr };
        let vftable = &unsafe { slice::from_raw_parts(instance.vtbl, 1) }[0];

        (vftable.fn_log)(
            log_ptr,
            format!("{}\0", category).as_str().as_ptr() as *const _,
            format!("{}\0", text).as_str().as_ptr() as *const _,
        );
    }

    #[macro_export]
    macro_rules! log {
        // log!("category", *format! args*);
        ($category:literal, $($arg:tt)*) => {
            crate::log::log($category, format!($($arg)*).as_str());
        };

        // log!(*format! args*);
        ($($arg:tt)*) => {
            crate::log::log(module_path!(), format!($($arg)*).as_str());
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
