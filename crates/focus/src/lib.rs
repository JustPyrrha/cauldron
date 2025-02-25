#![feature(once_cell_try)]
#![allow(static_mut_refs)]

use crate::renderer::msg_filter::MessageFilter;
use egui::RawInput;
use minhook::{MH_ApplyQueued, MH_Initialize, MH_STATUS, MH_Uninitialize, MhHook};
use std::cell::OnceCell;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};

pub mod hooks;
pub mod renderer;
pub mod util;

static mut SUNWING: OnceCell<Focus> = OnceCell::new();

pub trait EguiRenderLoop {
    fn initialize<'a>(&'a mut self, _ctx: &'a egui::Context) {}

    fn before_render<'a>(&'a mut self, _ctx: &'a egui::Context) {}

    fn render(&mut self, ctx: &egui::Context);

    fn on_wnd_proc(&self, _hwnd: HWND, _umsg: u32, _wparam: WPARAM, _lparam: LPARAM) {}

    fn message_filter(&self, _input: &RawInput) -> MessageFilter {
        MessageFilter::empty()
    }
}

pub trait Hooks {
    fn from_render_loop<T>(t: T) -> Box<Self>
    where
        Self: Sized,
        T: EguiRenderLoop + Send + Sync + 'static;
    fn hooks(&self) -> &[MhHook];
    unsafe fn unhook(&mut self);
}

pub struct Focus(Vec<Box<dyn Hooks>>);
unsafe impl Send for Focus {}
unsafe impl Sync for Focus {}

impl Focus {
    pub fn builder() -> FocusBuilder {
        FocusBuilder(Focus::new())
    }

    fn new() -> Self {
        match unsafe { MH_Initialize() } {
            MH_STATUS::MH_ERROR_ALREADY_INITIALIZED | MH_STATUS::MH_OK => {}
            status @ MH_STATUS::MH_ERROR_MEMORY_ALLOC => panic!("MH_Initialize: {status:?}"),
            _ => unreachable!(),
        }

        Focus(Vec::new())
    }

    fn hooks(&self) -> impl IntoIterator<Item = &MhHook> {
        self.0.iter().flat_map(|h| h.hooks())
    }

    pub fn apply(self) -> Result<(), MH_STATUS> {
        for hook in self.hooks() {
            unsafe { hook.queue_enable()? };
        }
        unsafe { MH_ApplyQueued().ok_context("MH_ApplyQueued")? };
        unsafe { SUNWING.set(self).ok() };
        Ok(())
    }

    pub fn unapply(&mut self) -> Result<(), MH_STATUS> {
        for hook in self.hooks() {
            unsafe { hook.queue_disable()? };
        }
        unsafe { MH_ApplyQueued().ok_context("MH_ApplyQueued")? };
        unsafe { MH_Uninitialize().ok_context("MH_Uninitialize")? };
        for hook in &mut self.0 {
            unsafe { hook.unhook() };
        }
        Ok(())
    }
}

pub struct FocusBuilder(Focus);

impl FocusBuilder {
    pub fn with<T: Hooks + 'static>(
        mut self,
        render_loop: impl EguiRenderLoop + Send + Sync + 'static,
    ) -> Self {
        self.0.0.push(T::from_render_loop(render_loop));
        self
    }

    pub fn build(self) -> Focus {
        self.0
    }
}
