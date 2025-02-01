#![feature(c_variadic)]
#![feature(once_cell_get_mut)]

#[doc(include = "../README.md")]
#[cfg(not(any(feature = "hfw")))]
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
}

#[cfg(feature = "nixxes")]
pub mod log {
    use crate::mem::offsets::Offsets;
    use crate::types::nixxes::log::NxLogImpl;
    use std::slice;

    pub fn log(category: &str, text: &str) {
        Offsets::instance().setup();
        let log_ptr = unsafe {
            *Offsets::instance()
                .resolve_t::<*mut NxLogImpl>("nx::NxLogImpl::Instance")
                .unwrap()
        };
        let instance = unsafe { &*log_ptr };
        let vftable = &unsafe { slice::from_raw_parts(instance.vftable, 1) }[0];

        (vftable.fn_log)(
            log_ptr,
            format!("{}\0", category).as_str().as_ptr() as *const _,
            format!("{}\0", text).as_str().as_ptr() as *const _,
        );
    }

    #[macro_export]
    macro_rules! log {
        // log!("hello world!");
        ($log:literal) => {
            crate::log::log(module_path!(), $log);
        };
        // log!("my category", "hello world!");
        ($category:literal, $log:literal) => {
            crate::log::log($category, $log);
        };
        //
        ($log:expr) => {
            crate::log::log(module_path!(), $log);
        };
    }
}
