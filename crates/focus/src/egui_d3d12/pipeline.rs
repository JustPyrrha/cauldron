use crate::RenderLoop;
use crate::egui_d3d12::input::{collect_input, process_input};
use crate::egui_d3d12::message_filter::MessageFilter;
use crate::egui_d3d12::painter::{Painter, split_output};
use egui::{Color32, Context, Event, RawInput};
use libdecima::log;
use once_cell::sync::OnceCell;
use once_cell::unsync::Lazy;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Instant;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Direct3D12::ID3D12Resource;
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, DefWindowProcW, GWLP_WNDPROC, SetWindowLongPtrW,
};
use windows::core::Result;

static mut PIPELINE_STATE: Lazy<Mutex<HashMap<usize, Arc<PipelineSharedState>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug)]
pub struct PipelineMessage {
    pub hwnd: usize,
    pub msg: u32,
    pub wparam: WPARAM,
    pub lparam: LPARAM,
}

pub struct PipelineSharedState {
    pub message_filter: AtomicU32,
    pub wnd_proc: unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT,
    pub tx: Sender<PipelineMessage>,
}

pub struct Pipeline {
    pub hwnd: usize,
    pub ctx: Context,
    pub painter: Painter,
    pub render_loop: RenderLoop,
    pub rx: Receiver<PipelineMessage>,
    pub shared_state: Arc<PipelineSharedState>,
    pub queue_buffer: OnceCell<Vec<PipelineMessage>>,
    #[allow(unused)]
    pub start_of_first_frame: OnceCell<Instant>,
    pub egui_events: Mutex<Vec<Event>>,
}

impl Pipeline {
    pub fn new(
        hwnd: HWND,
        mut ctx: Context,
        painter: Painter,
        mut render_loop: RenderLoop,
    ) -> std::result::Result<Self, (windows::core::Error, RenderLoop)> {
        // todo: init image loaders
        render_loop.initialize(&mut ctx);
        egui_extras::install_image_loaders(&mut ctx);
        catppuccin_egui::set_theme(&ctx, catppuccin_egui::MOCHA);
        let mut visuals = ctx.style().visuals.clone();
        let shadow = visuals.window_shadow.color;
        visuals.window_shadow.color =
            Color32::from_rgba_premultiplied(shadow.r(), shadow.g(), shadow.b(), 10);
        ctx.set_visuals(visuals);

        let wnd_proc = unsafe {
            std::mem::transmute(SetWindowLongPtrW(
                hwnd,
                GWLP_WNDPROC,
                pipeline_wnd_proc as _,
            ))
        };

        let (tx, rx) = channel();
        let shard_state = Arc::new(PipelineSharedState {
            message_filter: AtomicU32::new(0u32),
            wnd_proc,
            tx,
        });

        unsafe { PIPELINE_STATE.lock() }.insert(hwnd.0 as usize, Arc::clone(&shard_state));
        let queue_buffer = OnceCell::from(Vec::new());

        Ok(Self {
            hwnd: hwnd.0 as usize,
            ctx,
            painter,
            render_loop,
            rx,
            shared_state: Arc::clone(&shard_state),
            queue_buffer,
            start_of_first_frame: OnceCell::new(),
            egui_events: Mutex::new(Vec::new()),
        })
    }

    pub fn prepare(&mut self) -> Result<RawInput> {
        let mut queue_buf = self.queue_buffer.take().unwrap();
        queue_buf.clear();
        queue_buf.extend(self.rx.try_iter());
        queue_buf.drain(..).for_each(
            |PipelineMessage {
                 hwnd,
                 msg,
                 wparam,
                 lparam,
             }| {
                process_input(
                    HWND(hwnd as *mut _),
                    msg,
                    wparam,
                    lparam,
                    &self.egui_events,
                    &self.render_loop,
                );
            },
        );

        self.queue_buffer.set(queue_buf).expect("should be empty");

        let raw_input = collect_input(&self.egui_events, &mut self.ctx, HWND(self.hwnd as *mut _));
        self.egui_events.lock().clear();
        let message_filter = self.render_loop.message_filter(&raw_input);
        self.shared_state
            .message_filter
            .store(message_filter.bits(), Ordering::SeqCst);

        self.render_loop.before_render(&mut self.ctx);

        Ok(raw_input)
    }

    pub fn render(&mut self, raw_input: RawInput, resource: ID3D12Resource) -> Result<()> {
        let output = self.ctx.run(raw_input, |ctx| self.render_loop.render(ctx));
        let (render_output, ..) = split_output(output);

        self.painter
            .render(&mut self.ctx, render_output, resource)?;

        Ok(())
    }
}

unsafe extern "system" fn pipeline_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        let state = {
            let Some(guard) = PIPELINE_STATE.try_lock() else {
                return DefWindowProcW(hwnd, msg, wparam, lparam);
            };

            let Some(state) = guard.get(&(hwnd.0 as usize)) else {
                return DefWindowProcW(hwnd, msg, wparam, lparam);
            };

            Arc::clone(state)
        };

        if let Err(e) = state.tx.send(PipelineMessage {
            hwnd: hwnd.0 as usize,
            msg,
            wparam,
            lparam,
        }) {
            log!("pipeline error: {e:?}")
        };

        let filter = MessageFilter::from_bits_retain(state.message_filter.load(Ordering::SeqCst));

        if filter.is_blocking(msg) {
            LRESULT(1)
        } else {
            CallWindowProcW(Some(state.wnd_proc), hwnd, msg, wparam, lparam)
        }
    }
}
