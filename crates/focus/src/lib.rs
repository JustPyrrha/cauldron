#![allow(static_mut_refs)]

use crate::egui_d3d12::message_filter::MessageFilter;
use egui::RawInput;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};

mod editor;
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
    use crate::editor::{FocusEditor, widgets_demo_window::WidgetsDemo};
    use crate::egui_d3d12::message_filter::MessageFilter;
    use crate::egui_d3d12::painter::Painter;
    use crate::egui_d3d12::pipeline::Pipeline;
    use crate::egui_d3d12::util::print_dxgi_debug_messages;
    use egui::{Context, Key, RawInput};
    use glam::{EulerRot, Mat3};
    use libdecima::log;
    use libdecima::mem::offsets::Offset;
    use libdecima::types::decima::core::camera_entity::CameraEntity;
    use libdecima::types::decima::core::entity::Entity;
    use libdecima::types::decima::core::player::Player;
    use libdecima::types::decima::core::world_transform::GlamTransform;
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
        // log!("attach");
        // util::enable_debug_interface(false);

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
        // log!("focus::internal", "{:p}", present_ptr);

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
        // log!("attach complete");
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
            RENDER_LOOP.get_or_init(|| Box::new(FocusUI::new()));
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
    struct FocusUI {
        pub ui_open: bool,
        pub editor_open: bool,
        pub transforms_open: bool,
        pub widgets_open: bool,
        pub show_gizmos: bool,
        pub player_entity_ref: Option<&'static mut Entity>,
        pub player_camera_ref: Option<&'static mut CameraEntity>,
        pub editor: FocusEditor,
        pub widgets_demo: WidgetsDemo,
    }

    impl FocusUI {
        #[allow(dead_code)]
        pub fn new() -> Self {
            FocusUI {
                ui_open: false,
                editor_open: false,
                transforms_open: false,
                widgets_open: false,
                show_gizmos: false,
                player_entity_ref: None,
                player_camera_ref: None,
                editor: FocusEditor::default(),
                widgets_demo: WidgetsDemo::default(),
            }
        }
    }

    impl EguiRenderLoop for FocusUI {
        fn before_render<'a>(&'a mut self, ctx: &'a Context) {
            if ctx.input(|f| f.key_pressed(Key::Backtick)) {
                log!("toggle ui");
                self.ui_open = !self.ui_open;
            }
        }

        fn render(&mut self, ctx: &Context) {
            if self.transforms_open {
                egui::Window::new("Transforms")
                    .resizable(false)
                    .open(&mut self.transforms_open)
                    .show(ctx, |ui| {
                        if self.player_entity_ref.is_none() {
                            let player = Player::get_local_player(0);
                            let player = unsafe { &*player };
                            self.player_entity_ref = Some(unsafe { &mut *player.entity });
                            self.player_camera_ref =
                                Some(unsafe { &mut *player.get_last_active_camera().unwrap() });
                        }

                        if ui.button("get references").clicked() {
                            let player = Player::get_local_player(0);
                            let player = unsafe { &*player };
                            self.player_entity_ref = Some(unsafe { &mut *player.entity });
                            self.player_camera_ref =
                                Some(unsafe { &mut *player.get_last_active_camera().unwrap() });
                        }
                        ui.spacing();
                        ui.collapsing("Player", |ui| {
                            egui::Grid::new("player")
                                .num_columns(2)
                                .spacing([12.0, 8.0])
                                .striped(true)
                                .show(ui, |ui| {
                                    let transform =
                                        self.player_entity_ref.as_ref().unwrap().get_transform();
                                    ui.label("Position");
                                    ui.label(format!(
                                        "x: {:.4}, y: {:.4}, z: {:.4}",
                                        transform.pos.x, transform.pos.y, transform.pos.z
                                    ));
                                    ui.end_row();

                                    let (rot_x, rot_y, rot_z) =
                                        Mat3::from(transform.rot).to_euler(EulerRot::XYZ);
                                    ui.label("Rotation");
                                    ui.label(format!(
                                        "x: {:.4}, y: {:.4}, z: {:.4}",
                                        rot_x, rot_y, rot_z
                                    ));
                                    ui.end_row();
                                });
                        });
                        ui.spacing();
                        if let Some(camera) = &self.player_camera_ref {
                            ui.collapsing("Camera", |ui| {
                                egui::Grid::new("camera")
                                    .num_columns(2)
                                    .spacing([12.0, 8.0])
                                    .striped(true)
                                    .show(ui, |ui| {
                                        let mut transform =
                                            GlamTransform::from(camera.get_transform().clone());
                                        let player_transform = GlamTransform::from(
                                            self.player_entity_ref
                                                .as_ref()
                                                .unwrap()
                                                .get_transform()
                                                .clone(),
                                        );
                                        transform.pos += player_transform.pos;
                                        transform.rot += player_transform.rot;

                                        ui.label("Position");
                                        ui.label(format!(
                                            "x: {:.4}, y: {:.4}, z: {:.4}",
                                            transform.pos.x, transform.pos.y, transform.pos.z
                                        ));
                                        ui.end_row();

                                        let (rot_x, rot_y, rot_z) =
                                            Mat3::from(transform.rot).to_euler(EulerRot::XYZ);
                                        ui.label("Rotation");
                                        ui.label(format!(
                                            "x: {:.4}, y: {:.4}, z: {:.4}",
                                            rot_x, rot_y, rot_z
                                        ));
                                        ui.end_row();
                                    });
                            });
                        }
                    });
            }

            if !self.ui_open {
                return;
            }

            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.label(format!("Cauldron: v{}", env!("CARGO_PKG_VERSION")));
                    ui.separator();
                    ui.toggle_value(&mut self.editor_open, "Editor");
                    ui.toggle_value(&mut self.transforms_open, "Transforms");
                    ui.toggle_value(&mut self.widgets_open, "Widgets Demo");
                    ui.toggle_value(&mut self.show_gizmos, "Gizmos");
                });
            });

            egui::Window::new("Widgets Demo")
                .vscroll(true)
                .open(&mut self.widgets_open)
                .show(ctx, |ui| {
                    self.widgets_demo.ui(ui);
                });

            egui::Window::new("Editor")
                .vscroll(true)
                .open(&mut self.editor_open.clone())
                .show(ctx, |ui| {
                    self.editor.editor_ui(ui);
                });
        }

        fn message_filter(&self, _: &RawInput) -> MessageFilter {
            if self.ui_open {
                MessageFilter::all()
            } else {
                MessageFilter::empty()
            }
        }
    }

    unsafe impl Send for FocusUI {}
    unsafe impl Sync for FocusUI {}
}
