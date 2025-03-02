#[doc(hidden)]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
unsafe extern "system" fn DllMain(_: usize, reason: u32, _: isize) -> bool {
    unsafe {
        match reason {
            1 => {
                std::thread::spawn(|| cauldron::handle_dll_attach());
            }
            0 => {
                cauldron::handle_dll_detach();
            }
            _ => {}
        }
    }

    true
}
