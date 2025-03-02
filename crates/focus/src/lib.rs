#![allow(static_mut_refs)]

use crate::egui_d3d12::message_filter::MessageFilter;
use egui::RawInput;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};

pub mod egui_d3d12;

pub type RenderLoop = Box<dyn EguiRenderLoop + Send + Sync>;

pub trait EguiRenderLoop {
    fn initialize<'a>(&'a mut self, _ctx: &'a egui::Context) {}

    fn before_render<'a>(&'a mut self, _ctx: &'a egui::Context) {}

    fn render(&mut self, ctx: &egui::Context);

    fn on_wnd_proc(&self, _hwnd: HWND, _umsg: u32, _wparam: WPARAM, _lparam: LPARAM) {}

    fn message_filter(&self, _input: &RawInput) -> MessageFilter {
        MessageFilter::empty()
    }
}

#[doc(hidden)]
pub mod internal {
    use crate::EguiRenderLoop;
    use crate::egui_d3d12::painter::Painter;
    use crate::egui_d3d12::pipeline::Pipeline;
    use crate::egui_d3d12::util;
    use crate::egui_d3d12::util::print_dxgi_debug_messages;
    use egui::Context;
    use libdecima::log;
    use libdecima::mem::offsets::Offset;
    use libdecima::types::nixxes::nx_d3d::NxD3DImpl;
    use libdecima::types::nixxes::nx_dxgi::NxDXGIImpl;
    use minhook::{MH_ApplyQueued, MH_EnableHook, MH_Initialize, MH_STATUS, MhHook};
    use once_cell::sync::OnceCell;
    use parking_lot::Mutex;
    use std::ffi::c_void;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Direct3D12::{ID3D12CommandQueue, ID3D12Resource};
    use windows::Win32::Graphics::Dxgi::IDXGISwapChain3;
    use windows::core::{Error, HRESULT, Result};
    use windows_core::Interface;

    static PIPELINE: OnceCell<Mutex<Pipeline>> = OnceCell::new();
    static mut RENDER_LOOP: OnceCell<Box<dyn EguiRenderLoop + Send + Sync>> = OnceCell::new();
    static DXGI_PRESENT: OnceCell<extern "C" fn(this: *mut NxDXGIImpl, unk: *mut c_void) -> bool> =
        OnceCell::new();

    pub fn attach() {
        log!("attach");
        util::enable_debug_interface(false);

        match unsafe { MH_Initialize() } {
            MH_STATUS::MH_ERROR_ALREADY_INITIALIZED | MH_STATUS::MH_OK => {}
            status @ MH_STATUS::MH_ERROR_MEMORY_ALLOC => panic!("MH_Initialize: {status:?}"),
            _ => unreachable!(),
        }

        let present_ptr = Offset::from_signature(
            "48 8D 0D ? ? ? ? 66 89 68 08 48 89 08 40 88 68 0A 48 89 68 0C 48 89 68 18 48 89 68 20",
        )
        .unwrap()
        .as_relative(7)
        .as_adjusted(size_of::<*mut c_void>() * 10)
        .as_ptr::<*mut c_void>();
        log!("focus::internal", "{:p}", present_ptr);

        let present_hook =
            unsafe { MhHook::new(*present_ptr, present_hook_impl as *mut _).unwrap() };

        unsafe {
            DXGI_PRESENT
                .set(std::mem::transmute(present_hook.trampoline()))
                .unwrap();
        }

        unsafe {
            // enable all
            MH_EnableHook(std::ptr::null_mut())
                .ok()
                .expect("focus: failed to queue enable hooks");
            MH_ApplyQueued()
                .ok()
                .expect("focus: failed to apply queued hooks");
        };
        log!("attach complete");
    }

    fn init_pipeline(hwnd: HWND) -> Result<Mutex<Pipeline>> {
        let d3d = NxD3DImpl::get_instance().unwrap();
        let command_queue = NxD3DImpl::fn_get_command_queue(d3d as *const _ as *mut _, 0) as *mut _;
        let command_queue =
            unsafe { ID3D12CommandQueue::from_raw_borrowed(&command_queue).unwrap() };
        let ctx = Context::default();
        let painter = Painter::new(&command_queue)?;

        // todo: move this
        unsafe {
            RENDER_LOOP.get_or_init(|| Box::new(DefaultRenderLoop::new()));
        }
        let Some(render_loop) = (unsafe { RENDER_LOOP.take() }) else {
            return Err(Error::from(HRESULT(-1)));
        };
        let pipeline =
            Pipeline::new(hwnd, ctx, painter, render_loop).map_err(|(e, render_loop)| {
                unsafe { RENDER_LOOP.get_or_init(move || render_loop) };
                e
            })?;
        Ok(Mutex::new(pipeline))
    }

    #[allow(dead_code)]
    fn render(swap_chain: &IDXGISwapChain3) -> Result<()> {
        let pipeline =
            PIPELINE.get_or_try_init(|| init_pipeline(unsafe { swap_chain.GetHwnd()? }))?;
        let Some(mut pipeline) = pipeline.try_lock() else {
            return Err(Error::from(HRESULT(-1)));
        };
        let input = pipeline.prepare()?;
        let target: ID3D12Resource =
            unsafe { swap_chain.GetBuffer(swap_chain.GetCurrentBackBufferIndex())? };
        pipeline.render(input, target)?;
        Ok(())
    }

    fn present_hook_impl(dxgi: *mut NxDXGIImpl, unk: *mut c_void) -> bool {
        unsafe {
            let instance = &*dxgi;
            if instance.initialized {
                let swap_chain = instance.swap_chain as *mut _;
                let swap_chain = &IDXGISwapChain3::from_raw_borrowed(&swap_chain).unwrap();

                if let Err(e) = render(swap_chain) {
                    print_dxgi_debug_messages();
                    log!("Render error: {e}");
                }
            }
        }

        (DXGI_PRESENT.get().unwrap())(dxgi, unk)
    }

    #[derive(Debug)]
    #[allow(dead_code)]
    struct DefaultRenderLoop {}

    impl DefaultRenderLoop {
        #[allow(dead_code)]
        pub fn new() -> Self {
            DefaultRenderLoop {}
        }
    }

    impl EguiRenderLoop for DefaultRenderLoop {
        fn initialize<'a>(&'a mut self, _ctx: &'a Context) {}

        fn render(&mut self, ctx: &Context) {
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.label(format!("Cauldron: v{}", env!("CARGO_PKG_VERSION")));
                    ui.separator();
                });
            });

            egui::Window::new("🔧 Settings")
                .vscroll(true)
                .show(ctx, |ui| {
                    ctx.settings_ui(ui);
                });

            egui::Window::new("🔍 Inspection")
                .vscroll(true)
                .show(ctx, |ui| {
                    ctx.inspection_ui(ui);
                });

            egui::Window::new("📝 Memory")
                .resizable(false)
                .show(ctx, |ui| {
                    ctx.memory_ui(ui);
                });
        }
    }

    unsafe impl Send for DefaultRenderLoop {}
    unsafe impl Sync for DefaultRenderLoop {}
}
