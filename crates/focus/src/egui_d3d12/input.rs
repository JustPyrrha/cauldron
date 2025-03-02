use crate::RenderLoop;
use egui::{
    Context, Event, Key, Modifiers, MouseWheelUnit, PointerButton, Pos2, RawInput, Rect, Vec2,
};
use parking_lot::Mutex;
use std::ffi::CStr;
use windows::Wdk::System::SystemInformation::NtQuerySystemTime;
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::{
    Foundation::{HWND, RECT},
    System::{
        DataExchange::{CloseClipboard, GetClipboardData, OpenClipboard},
        Ole::CF_TEXT,
        SystemServices::{MK_CONTROL, MK_SHIFT},
    },
    UI::{
        Input::KeyboardAndMouse::{
            GetAsyncKeyState, VIRTUAL_KEY, VK_BACK, VK_CONTROL, VK_DELETE, VK_DOWN, VK_END,
            VK_ESCAPE, VK_HOME, VK_INSERT, VK_LEFT, VK_LSHIFT, VK_NEXT, VK_OEM_3, VK_PRIOR,
            VK_RETURN, VK_RIGHT, VK_SPACE, VK_TAB, VK_UP,
        },
        WindowsAndMessaging::{
            GetClientRect, WHEEL_DELTA, WM_CHAR, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDBLCLK,
            WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDBLCLK, WM_MBUTTONDOWN, WM_MBUTTONUP,
            WM_MOUSEHWHEEL, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_RBUTTONDBLCLK, WM_RBUTTONDOWN,
            WM_RBUTTONUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
        },
    },
};

pub(crate) fn process_input(
    hwnd: HWND,
    umsg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    events: &Mutex<Vec<Event>>,
    render_loop: &RenderLoop,
) {
    match umsg {
        WM_MOUSEMOVE => {
            events.lock().push(Event::PointerMoved(get_pos(lparam)));
        }
        WM_LBUTTONDOWN | WM_LBUTTONDBLCLK => {
            events.lock().push(Event::PointerButton {
                pos: get_pos(lparam),
                button: PointerButton::Primary,
                pressed: true,
                modifiers: get_modifiers(wparam),
            });
        }
        WM_LBUTTONUP => {
            events.lock().push(Event::PointerButton {
                pos: get_pos(lparam),
                button: PointerButton::Primary,
                pressed: false,
                modifiers: get_modifiers(wparam),
            });
        }
        WM_RBUTTONDOWN | WM_RBUTTONDBLCLK => {
            events.lock().push(Event::PointerButton {
                pos: get_pos(lparam),
                button: PointerButton::Secondary,
                pressed: true,
                modifiers: get_modifiers(wparam),
            });
        }
        WM_RBUTTONUP => {
            events.lock().push(Event::PointerButton {
                pos: get_pos(lparam),
                button: PointerButton::Secondary,
                pressed: false,
                modifiers: get_modifiers(wparam),
            });
        }
        WM_MBUTTONDOWN | WM_MBUTTONDBLCLK => {
            events.lock().push(Event::PointerButton {
                pos: get_pos(lparam),
                button: PointerButton::Middle,
                pressed: true,
                modifiers: get_modifiers(wparam),
            });
        }
        WM_MBUTTONUP => {
            events.lock().push(Event::PointerButton {
                pos: get_pos(lparam),
                button: PointerButton::Middle,
                pressed: false,
                modifiers: get_modifiers(wparam),
            });
        }
        WM_CHAR => {
            if let Some(ch) = char::from_u32(wparam.0 as _) {
                if !ch.is_control() {
                    events.lock().push(Event::Text(ch.into()));
                }
            }
        }
        WM_MOUSEWHEEL => {
            events.lock().push(Event::MouseWheel {
                unit: MouseWheelUnit::Point,
                modifiers: get_modifiers(wparam),
                delta: Vec2 {
                    x: 0.0,
                    y: (wparam.0 >> 16) as i16 as f32 * 10. / WHEEL_DELTA as f32,
                },
            });
        }
        WM_MOUSEHWHEEL => {
            events.lock().push(Event::MouseWheel {
                unit: MouseWheelUnit::Point,
                modifiers: get_modifiers(wparam),
                delta: Vec2 {
                    x: (wparam.0 >> 16) as i16 as f32 * 10. / WHEEL_DELTA as f32,
                    y: 0.0,
                },
            });
        }
        msg @ (WM_KEYDOWN | WM_SYSKEYDOWN) => {
            if let Some(key) = get_key(wparam) {
                let lock = &mut *events.lock();
                let mods = get_key_modifiers(msg);

                if key == Key::Space {
                    lock.push(Event::Text(String::from(" ")));
                } else if key == Key::V && mods.ctrl {
                    if let Some(clipboard) = get_clipboard_text() {
                        lock.push(Event::Text(clipboard));
                    }
                } else if key == Key::C && mods.ctrl {
                    lock.push(Event::Copy);
                } else if key == Key::X && mods.ctrl {
                    lock.push(Event::Cut);
                } else {
                    lock.push(Event::Key {
                        key,
                        pressed: true,
                        repeat: false,
                        physical_key: None,
                        modifiers: get_key_modifiers(msg),
                    });
                }
            }
        }
        msg @ (WM_KEYUP | WM_SYSKEYUP) => {
            if let Some(key) = get_key(wparam) {
                events.lock().push(Event::Key {
                    key,
                    pressed: false,
                    repeat: false,
                    physical_key: None,
                    modifiers: get_key_modifiers(msg),
                });
            }
        }
        _ => {}
    }

    render_loop.on_wnd_proc(hwnd, umsg, wparam, lparam);
}

pub fn collect_input(events: &Mutex<Vec<Event>>, ctx: &mut Context, hwnd: HWND) -> RawInput {
    let events = events.lock().clone();
    RawInput {
        viewport_id: ctx.viewport_id(),
        screen_rect: Some(get_screen_rect(hwnd)),
        time: Some(get_system_time()),
        modifiers: Modifiers::default(),
        max_texture_side: None,
        predicted_dt: 1. / 60.,
        hovered_files: Vec::new(),
        dropped_files: Vec::new(),
        focused: false,
        events,
        system_theme: None,
        ..Default::default()
    }
}

pub fn get_system_time() -> f64 {
    let mut time = 0;
    unsafe {
        NtQuerySystemTime(&mut time)
            .ok()
            .expect("NtQuerySystemTime failed");
    }
    // `NtQuerySystemTime` returns how many 100 nanosecond intervals
    // past since 1st Jan, 1601.
    (time as f64) / 10_000_000.
}

#[inline]
pub fn get_screen_size(hwnd: HWND) -> Pos2 {
    let mut rect = RECT::default();
    unsafe {
        GetClientRect(hwnd, &mut rect).expect("GetClientRect");
    }

    Pos2::new(
        (rect.right - rect.left) as f32,
        (rect.bottom - rect.top) as f32,
    )
}

#[inline]
pub fn get_screen_rect(hwnd: HWND) -> Rect {
    Rect {
        min: Pos2::ZERO,
        max: get_screen_size(hwnd),
    }
}

fn get_pos(lparam: LPARAM) -> Pos2 {
    let x = (lparam.0 & 0xFFFF) as i16 as f32;
    let y = (lparam.0 >> 16 & 0xFFFF) as i16 as f32;

    Pos2::new(x, y)
}

fn get_modifiers(wparam: WPARAM) -> Modifiers {
    Modifiers {
        alt: false,
        ctrl: (wparam.0 & MK_CONTROL.0 as usize) != 0,
        shift: (wparam.0 & MK_SHIFT.0 as usize) != 0,
        mac_cmd: false,
        command: (wparam.0 & MK_CONTROL.0 as usize) != 0,
    }
}

fn get_key_modifiers(msg: u32) -> Modifiers {
    let ctrl = unsafe { GetAsyncKeyState(VK_CONTROL.0 as _) != 0 };
    let shift = unsafe { GetAsyncKeyState(VK_LSHIFT.0 as _) != 0 };

    Modifiers {
        alt: msg == WM_SYSKEYDOWN,
        mac_cmd: false,
        command: ctrl,
        shift,
        ctrl,
    }
}

//todo(py): this is still missing quite a lot
fn get_key(wparam: WPARAM) -> Option<Key> {
    match wparam.0 {
        0x30..=0x39 => unsafe { Some(std::mem::transmute::<_, Key>(wparam.0 as u8 - 0x21)) },
        0x41..=0x5A => unsafe { Some(std::mem::transmute::<_, Key>(wparam.0 as u8 - 0x28)) },
        _ => match VIRTUAL_KEY(wparam.0 as u16) {
            VK_DOWN => Some(Key::ArrowDown),
            VK_LEFT => Some(Key::ArrowLeft),
            VK_RIGHT => Some(Key::ArrowRight),
            VK_UP => Some(Key::ArrowUp),
            VK_ESCAPE => Some(Key::Escape),
            VK_TAB => Some(Key::Tab),
            VK_BACK => Some(Key::Backspace),
            VK_RETURN => Some(Key::Enter),
            VK_SPACE => Some(Key::Space),
            VK_INSERT => Some(Key::Insert),
            VK_DELETE => Some(Key::Delete),
            VK_HOME => Some(Key::Home),
            VK_END => Some(Key::End),
            VK_PRIOR => Some(Key::PageUp),
            VK_NEXT => Some(Key::PageDown),
            VK_OEM_3 => Some(Key::Backtick),
            _ => None,
        },
    }
}

fn get_clipboard_text() -> Option<String> {
    unsafe {
        if OpenClipboard(None).is_ok() {
            let txt = GetClipboardData(CF_TEXT.0 as u32)
                .expect("GetClipboardData")
                .0 as *const i8;
            let data = Some(CStr::from_ptr(txt).to_str().ok()?.to_string());
            CloseClipboard().expect("CloseClipboard");
            data
        } else {
            None
        }
    }
}
