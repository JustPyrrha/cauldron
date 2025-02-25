#[doc(hidden)]
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
unsafe extern "system" fn DllMain(_: usize, reason: u32, _: isize) -> bool {
    match reason {
        1 => {
            std::thread::spawn(|| unsafe { cauldron::handle_dll_attach() });
        }
        _ => {}
    }

    true
}
