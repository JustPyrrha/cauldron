#[doc(include = "../README.md")]

#[cfg(not(any(feature = "hfw")))]
compile_error!("At least one game target feature must be enabled.");

#[cfg(feature = "hfw")]
include!(concat!(env!("OUT_DIR"), "/hfw.rs"));
