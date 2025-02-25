use windows::Win32::UI::WindowsAndMessaging::{MESSAGEBOX_STYLE, MessageBoxW};
use windows::core::{HSTRING, PCWSTR};

pub(crate) fn message_box(title: &str, text: &str, icon: MESSAGEBOX_STYLE) {
    unsafe {
        MessageBoxW(
            None,
            PCWSTR::from_raw(HSTRING::from(text).as_ptr()),
            PCWSTR::from_raw(HSTRING::from(title).as_ptr()),
            icon,
        );
    }
}
