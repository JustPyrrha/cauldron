use crate::renderer::backend::dx12;
use crate::renderer::backend::dx12::{split_output, D3D12RenderEngine};
use crate::renderer::input::{collect_input, process_input};
use crate::renderer::msg_filter::MessageFilter;
use crate::renderer::RenderEngine;
use crate::EguiRenderLoop;
use egui::{Context, Event, RawInput};
use log::{debug, error, trace};
use once_cell::unsync::Lazy;
use parking_lot::Mutex;
use std::cell::OnceCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::mem;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{atomic, Arc};
use std::time::Instant;
use windows::core::{Error, Result};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Direct3D12::ID3D12Resource;
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, DefWindowProcW, SetWindowLongPtrW, GWLP_USERDATA, GWLP_WNDPROC,
};

type RenderLoop = Box<dyn EguiRenderLoop + Send + Sync>;
pub type FnWndProc =
    unsafe extern "system" fn(hwnd: HWND, umsg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT;

static mut PIPELINE_STATES: Lazy<Mutex<HashMap<isize, Arc<PipelineSharedState>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug)]
pub(crate) struct PipelineMessage(
    pub(crate) HWND,
    pub(crate) u32,
    pub(crate) WPARAM,
    pub(crate) LPARAM,
);

pub(crate) struct PipelineSharedState {
    pub(crate) message_filter: AtomicU32,
    pub(crate) wnd_proc: FnWndProc,
    pub(crate) tx: Sender<PipelineMessage>,
}

pub(crate) struct Pipeline {
    pub(crate) hwnd: isize,
    pub(crate) ctx: Context,
    engine: D3D12RenderEngine,
    render_loop: RenderLoop,
    rx: Receiver<PipelineMessage>,
    shared_state: Arc<PipelineSharedState>,
    queue_buffer: OnceCell<Vec<PipelineMessage>>,
    start_of_first_frame: OnceCell<Instant>,
    pub(crate) egui_events: Mutex<Vec<Event>>,
}

impl Pipeline {
    pub(crate) fn new(
        hwnd: isize,
        mut ctx: Context,
        mut engine: D3D12RenderEngine,
        mut render_loop: RenderLoop,
    ) -> std::result::Result<Self, (Error, RenderLoop)> {
        egui_extras::install_image_loaders(&mut ctx);
        render_loop.initialize(&mut ctx);

        let wnd_proc = unsafe {
            mem::transmute::<isize, FnWndProc>(SetWindowLongPtrW(
                HWND(hwnd as *mut c_void),
                GWLP_WNDPROC,
                pipeline_wnd_proc as usize as _,
            ))
        };

        let (tx, rx) = channel();
        let shared_state = Arc::new(PipelineSharedState {
            message_filter: AtomicU32::new(MessageFilter::empty().bits()),
            wnd_proc,
            tx,
        });

        unsafe { PIPELINE_STATES.lock() }.insert(hwnd, Arc::clone(&shared_state));
        let queue_buffer = OnceCell::from(Vec::new());

        Ok(Self {
            hwnd,
            ctx,
            engine,
            render_loop,
            rx,
            shared_state: Arc::clone(&shared_state),
            queue_buffer,
            start_of_first_frame: OnceCell::new(),
            egui_events: Mutex::new(Vec::new()),
        })
    }

    pub(crate) fn prepare_render(&mut self) -> Result<RawInput> {
        let mut queue_buffer = self.queue_buffer.take().unwrap();
        queue_buffer.clear();
        queue_buffer.extend(self.rx.try_iter());
        queue_buffer
            .drain(..)
            .for_each(|PipelineMessage(hwnd, umsg, wparam, lparam)| {
                process_input(hwnd, umsg, wparam.0, lparam.0, self);
            });
        self.queue_buffer
            .set(queue_buffer)
            .expect("OnceCell should be empty");
        let raw_input = collect_input(self);
        let message_filter = self.render_loop.message_filter(&raw_input);
        self.shared_state
            .message_filter
            .store(message_filter.bits(), Ordering::SeqCst);

        self.render_loop
            .before_render(&mut self.ctx);

        Ok(raw_input)
    }

    pub(crate) fn render(
        &mut self,
        raw_input: RawInput,
        render_target: ID3D12Resource,
    ) -> Result<()> {
        let output = self.ctx.run(raw_input, |ctx| self.render_loop.render(ctx));
        let (renderer_output, platform_output, _) = split_output(output);
        self.engine
            .render(&mut self.ctx, renderer_output, render_target)?;
        Ok(())
    }

    pub(crate) fn render_loop(&mut self) -> &mut RenderLoop {
        &mut self.render_loop
    }
}

unsafe extern "system" fn pipeline_wnd_proc(
    hwnd: isize,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> LRESULT {
    let shared_state = {
        let Some(guard) = PIPELINE_STATES.try_lock() else {
            error!("Could not lock shared state in window procedure");
            return DefWindowProcW(
                HWND(hwnd as *mut c_void),
                msg,
                WPARAM(wparam),
                LPARAM(lparam),
            );
        };

        let Some(shared_state) = guard.get(&hwnd) else {
            error!("Could not get shared state for handle {hwnd:?}");
            return DefWindowProcW(
                HWND(hwnd as *mut c_void),
                msg,
                WPARAM(wparam),
                LPARAM(lparam),
            );
        };
        Arc::clone(shared_state)
    };

    if let Err(e) = shared_state.tx.send(PipelineMessage(
        HWND(hwnd as *mut c_void),
        msg,
        WPARAM(wparam),
        LPARAM(lparam),
    )) {
        error!("Could not send window message through pipeline: {e:?}");
    }

    let message_filter =
        MessageFilter::from_bits_retain(shared_state.message_filter.load(Ordering::SeqCst));

    if message_filter.is_blocking(msg) {
        LRESULT(1)
    } else {
        CallWindowProcW(
            Some(shared_state.wnd_proc),
            HWND(hwnd as *mut c_void),
            msg,
            WPARAM(wparam),
            LPARAM(lparam),
        )
    }
}
