use log::debug;
use std::mem;
use windows::core::w;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, RegisterClassExW, CS_HREDRAW, CS_VREDRAW, WNDCLASSEXW,
    WS_EX_OVERLAPPEDWINDOW, WS_OVERLAPPEDWINDOW,
};

pub mod dx12;

/// A RAII dummy window.
///
/// Registers a class and creates a window on instantiation.
/// Destroys the window and unregisters the class on drop.
#[allow(dead_code)]
pub struct DummyHwnd(HWND, WNDCLASSEXW);

impl Default for DummyHwnd {
    fn default() -> Self {
        Self::new()
    }
}

impl DummyHwnd {
    /// Construct the dummy [`HWND`].
    pub fn new() -> Self {
        // The window procedure for the class just calls `DefWindowProcW`.
        unsafe extern "system" fn wnd_proc(
            hwnd: HWND,
            msg: u32,
            wparam: WPARAM,
            lparam: LPARAM,
        ) -> LRESULT { unsafe {
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }}

        // Create and register the class.
        let wndclass = WNDCLASSEXW {
            cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: unsafe { GetModuleHandleW(None).unwrap().into() },
            lpszClassName: w!("SUNWING"),
            ..Default::default()
        };
        debug!("{:?}", wndclass);
        unsafe { RegisterClassExW(&wndclass) };

        // Create the window.
        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_OVERLAPPEDWINDOW,
                wndclass.lpszClassName,
                w!("SUNWING"),
                WS_OVERLAPPEDWINDOW,
                0,
                0,
                3440,
                1440, //todo(py): get these at runtime
                None,
                None,
                Some(wndclass.hInstance),
                None,
            )
            .unwrap()
        };
        debug!("{:?}", hwnd);

        Self(hwnd, wndclass)
    }

    /// Retrieve the window handle.
    pub fn hwnd(&self) -> HWND {
        self.0
    }
}
