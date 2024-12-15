use crate::hooks::DummyHwnd;
use crate::renderer::backend::dx12::D3D12RenderEngine;
use crate::renderer::pipeline::Pipeline;
use crate::{util, EguiRenderLoop, Hooks};
use egui::Context;
use log::{debug, error, trace, warn};
use minhook::MhHook;
use once_cell::unsync::OnceCell;
use parking_lot::Mutex;
use std::ffi::c_void;
use std::sync::OnceLock;
use windows::core::{Error, Interface, Result, HRESULT};
use windows::Win32::Foundation::BOOL;
use windows::Win32::Graphics::Direct3D::D3D_FEATURE_LEVEL_11_0;
use windows::Win32::Graphics::Direct3D12::{
    D3D12CreateDevice, ID3D12CommandList, ID3D12CommandQueue, ID3D12Device, ID3D12Resource,
    D3D12_COMMAND_LIST_TYPE_DIRECT, D3D12_COMMAND_QUEUE_DESC, D3D12_COMMAND_QUEUE_FLAG_NONE,
};
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_FORMAT, DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_MODE_DESC, DXGI_MODE_SCALING_UNSPECIFIED,
    DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED, DXGI_RATIONAL, DXGI_SAMPLE_DESC,
};
use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory2, IDXGIFactory2, IDXGISwapChain, IDXGISwapChain3, DXGI_CREATE_FACTORY_FLAGS,
    DXGI_SWAP_CHAIN_DESC, DXGI_SWAP_CHAIN_FLAG_ALLOW_MODE_SWITCH, DXGI_SWAP_EFFECT_FLIP_DISCARD,
    DXGI_USAGE_RENDER_TARGET_OUTPUT,
};

type FnDXGISwapChainPresent =
    unsafe extern "system" fn(this: IDXGISwapChain3, sync_interval: u32, flags: u32) -> HRESULT;

type FnDXGISwapChainResizeBuffers = unsafe extern "system" fn(
    this: IDXGISwapChain3,
    buffer_count: u32,
    width: u32,
    height: u32,
    new_format: DXGI_FORMAT,
    flags: u32,
) -> HRESULT;

type FnD3D12CommandQueueExecuteCommandLists = unsafe extern "system" fn(
    this: ID3D12CommandQueue,
    num_command_lists: u32,
    command_lists: *mut ID3D12CommandList,
);

struct Trampolines {
    dxgi_swap_chain_present: FnDXGISwapChainPresent,
    dxgi_swap_chain_resize_buffers: FnDXGISwapChainResizeBuffers,
    d3d12_command_queue_execute_command_lists: FnD3D12CommandQueueExecuteCommandLists,
}

static mut TRAMPOLINES: OnceLock<Trampolines> = OnceLock::new();

enum InitializationContext {
    Empty,
    WithSwapChain(IDXGISwapChain3),
    Complete(IDXGISwapChain3, ID3D12CommandQueue),
    Done,
}

impl InitializationContext {
    fn insert_swap_chain(&mut self, swap_chain: &IDXGISwapChain3) {
        *self = match std::mem::replace(self, InitializationContext::Empty) {
            InitializationContext::Empty => {
                InitializationContext::WithSwapChain(swap_chain.clone())
            }
            s => s,
        }
    }

    fn insert_command_queue(&mut self, queue: &ID3D12CommandQueue) {
        *self = match std::mem::replace(self, InitializationContext::Empty) {
            InitializationContext::WithSwapChain(swap_chain) => {
                if unsafe { Self::check_command_queue(&swap_chain, queue) } {
                    trace!("found command queue matching {swap_chain:?} at {queue:?}");
                    InitializationContext::Complete(swap_chain, queue.clone())
                } else {
                    InitializationContext::WithSwapChain(swap_chain)
                }
            }
            s => s,
        }
    }

    fn get(&self) -> Option<(IDXGISwapChain3, ID3D12CommandQueue)> {
        if let InitializationContext::Complete(swap_chain, queue) = self {
            Some((swap_chain.clone(), queue.clone()))
        } else {
            None
        }
    }

    fn done(&mut self) {
        if let InitializationContext::Complete(..) = self {
            *self = InitializationContext::Done;
        }
    }

    unsafe fn check_command_queue(
        swap_chain: &IDXGISwapChain3,
        queue: &ID3D12CommandQueue,
    ) -> bool {
        let swap_chain_ptr = swap_chain.as_raw() as *mut *mut c_void;
        let readable = util::readable_region(swap_chain_ptr, 512);

        match readable.iter().position(|&ptr| ptr == queue.as_raw()) {
            Some(idx) => {
                debug!(
                    "Found command queue pointer in swap chain struct at offset +0x{:x}",
                    idx * size_of::<usize>()
                );
                true
            }
            None => {
                warn!("Couldn't find command queue in swap chain struct ({} of 512 pointers were readable)", readable.len());
                false
            }
        }
    }
}

static INITIALIZATION_CONTEXT: Mutex<InitializationContext> =
    Mutex::new(InitializationContext::Empty);
static mut PIPELINE: OnceCell<Mutex<Pipeline>> = OnceCell::new();
static mut RENDER_LOOP: OnceCell<Box<dyn EguiRenderLoop + Send + Sync>> = OnceCell::new();

unsafe fn init_pipeline() -> Result<Mutex<Pipeline>> {
    let Some((swap_chain, command_queue)) = ({ INITIALIZATION_CONTEXT.lock().get() }) else {
        error!("Initialization context not initialized");
        return Err(Error::from_hresult(HRESULT(-1)));
    };

    let hwnd = swap_chain.GetDesc()?.OutputWindow;
    let mut ctx = Context::default();
    let engine = D3D12RenderEngine::new(&command_queue, &mut ctx)?;

    let Some(render_loop) = RENDER_LOOP.take() else {
        error!("Render loop not initialized");
        return Err(Error::from_hresult(HRESULT(-1)));
    };

    {
        INITIALIZATION_CONTEXT.lock().done()
    }

    let pipeline =
        Pipeline::new(hwnd.0 as isize, ctx, engine, render_loop).map_err(|(e, rl)| {
            RENDER_LOOP.get_or_init(move || rl);
            e
        })?;

    Ok(Mutex::new(pipeline))
}

fn render(swap_chain: &IDXGISwapChain3) -> Result<()> {
    unsafe {
        let pipeline = PIPELINE.get_or_try_init(|| init_pipeline())?;
        let Some(mut pipeline) = pipeline.try_lock() else {
            error!("could not lock pipeline");
            return Err(Error::from_hresult(HRESULT(-1)));
        };

        let egui_input = pipeline.prepare_render()?;
        let target: ID3D12Resource =
            swap_chain.GetBuffer(swap_chain.GetCurrentBackBufferIndex())?;
        pipeline.render(egui_input, target)?;
    }

    Ok(())
}

unsafe extern "system" fn dxgi_swap_chain_present_impl(
    swap_chain: IDXGISwapChain3,
    sync_interval: u32,
    flags: u32,
) -> HRESULT {
    {
        INITIALIZATION_CONTEXT.lock().insert_swap_chain(&swap_chain);
    }

    let Trampolines {
        dxgi_swap_chain_present,
        ..
    } = TRAMPOLINES.get().expect("dx12 trampolines not initialized");

    if let Err(e) = render(&swap_chain) {
        util::print_dxgi_debug_messages();
        error!("render error: {e:?}");
    }
    trace!("Call IDXGISwapChain::Present trampoline");
    dxgi_swap_chain_present(swap_chain, sync_interval, flags)
}

unsafe extern "system" fn dxgi_swap_chain_resize_buffers_impl(
    this: IDXGISwapChain3,
    buffer_count: u32,
    width: u32,
    height: u32,
    new_format: DXGI_FORMAT,
    flags: u32,
) -> HRESULT {
    let Trampolines {
        dxgi_swap_chain_resize_buffers,
        ..
    } = TRAMPOLINES.get().expect("dx12 trampolines not initialized");
    trace!("Call IDXGISwapChain::ResizeBuffers trampoline");
    dxgi_swap_chain_resize_buffers(this, buffer_count, width, height, new_format, flags)
}

unsafe extern "system" fn d3d12_command_queue_execute_command_lists_impl(
    command_queue: ID3D12CommandQueue,
    num_command_lists: u32,
    command_lists: *mut ID3D12CommandList,
) {
    trace!("ID3D12::ExecuteCommandLists({command_queue:?}, {num_command_lists}, {command_lists:?}) invoked.");

    {
        INITIALIZATION_CONTEXT
            .lock()
            .insert_command_queue(&command_queue);
    }

    let Trampolines {
        d3d12_command_queue_execute_command_lists,
        ..
    } = TRAMPOLINES
        .get()
        .expect("d3d12 trampolines not initialized");

    d3d12_command_queue_execute_command_lists(command_queue, num_command_lists, command_lists);
}

fn get_target_addrs() -> (
    FnDXGISwapChainPresent,
    FnDXGISwapChainResizeBuffers,
    FnD3D12CommandQueueExecuteCommandLists,
) {
    let dummy_hwnd = DummyHwnd::new();

    let factory: IDXGIFactory2 =
        unsafe { CreateDXGIFactory2(DXGI_CREATE_FACTORY_FLAGS(0)) }.unwrap();


    let adapter = unsafe { factory.EnumAdapters(0) }.unwrap();

    let device: ID3D12Device =
        util::try_out_ptr(|v| unsafe { D3D12CreateDevice(&adapter, D3D_FEATURE_LEVEL_11_0, v) })
            .expect("failed to create device");


    let command_queue: ID3D12CommandQueue = unsafe {
        device.CreateCommandQueue(&D3D12_COMMAND_QUEUE_DESC {
            Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
            Priority: 0,
            Flags: D3D12_COMMAND_QUEUE_FLAG_NONE,
            NodeMask: 0,
        })
    }
    .unwrap();

    let swap_chain: IDXGISwapChain = match util::try_out_ptr(|v| unsafe {
        factory
            .CreateSwapChain(
                &command_queue,
                &DXGI_SWAP_CHAIN_DESC {
                    BufferDesc: DXGI_MODE_DESC {
                        Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                        ScanlineOrdering: DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED,
                        Scaling: DXGI_MODE_SCALING_UNSPECIFIED,
                        Width: 640,
                        Height: 480,
                        RefreshRate: DXGI_RATIONAL {
                            Numerator: 60,
                            Denominator: 1,
                        },
                    },
                    BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                    BufferCount: 2,
                    OutputWindow: dummy_hwnd.hwnd(),
                    Windowed: BOOL(1),
                    SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
                    SampleDesc: DXGI_SAMPLE_DESC {
                        Count: 1,
                        Quality: 0,
                    },
                    Flags: DXGI_SWAP_CHAIN_FLAG_ALLOW_MODE_SWITCH.0 as _,
                },
                v,
            )
            .ok()
    }) {
        Ok(swap_chain) => swap_chain,
        Err(e) => {
            util::print_dxgi_debug_messages();
            panic!("{e:?}");
        }
    };

    let present_ptr: FnDXGISwapChainPresent =
        unsafe { std::mem::transmute(swap_chain.vtable().Present) };
    let resize_buffers_ptr: FnDXGISwapChainResizeBuffers =
        unsafe { std::mem::transmute(swap_chain.vtable().ResizeBuffers) };
    let execute_lists_ptr: FnD3D12CommandQueueExecuteCommandLists =
        unsafe { std::mem::transmute(command_queue.vtable().ExecuteCommandLists) };

    (present_ptr, resize_buffers_ptr, execute_lists_ptr)
}

pub struct Dx12Hooks([MhHook; 3]);
impl Dx12Hooks {
    pub unsafe fn new<T>(t: T) -> Self
    where
        T: EguiRenderLoop + Send + Sync + 'static,
    {
        let (
            dxgi_swap_chain_present_addr,
            dxgi_swap_chain_resize_buffers_addr,
            d3d12_command_queue_execute_command_lists_addr,
        ) = get_target_addrs();

        trace!(
            "IDXGISwapChain::Present = {:p}",
            dxgi_swap_chain_present_addr as *const c_void
        );
        let hook_present = MhHook::new(
            dxgi_swap_chain_present_addr as *mut _,
            dxgi_swap_chain_present_impl as *mut _,
        )
        .expect("couldn't create IDXGISwapChain::Present hook");
        let hook_resize_buffers = MhHook::new(
            dxgi_swap_chain_resize_buffers_addr as *mut _,
            dxgi_swap_chain_resize_buffers_impl as *mut _,
        )
        .expect("couldn't create IDXGISwapChain::ResizeBuffers hook");
        let hook_cqecl = MhHook::new(
            d3d12_command_queue_execute_command_lists_addr as *mut _,
            d3d12_command_queue_execute_command_lists_impl as *mut _,
        )
        .expect("couldn't create ID3D12CommandQueue::ExecuteCommandLists hook");

        RENDER_LOOP.get_or_init(|| Box::new(t));

        TRAMPOLINES.get_or_init(|| Trampolines {
            dxgi_swap_chain_present: std::mem::transmute::<*mut c_void, FnDXGISwapChainPresent>(
                hook_present.trampoline(),
            ),
            dxgi_swap_chain_resize_buffers: std::mem::transmute::<
                *mut c_void,
                FnDXGISwapChainResizeBuffers,
            >(hook_resize_buffers.trampoline()),
            d3d12_command_queue_execute_command_lists: std::mem::transmute::<
                *mut c_void,
                FnD3D12CommandQueueExecuteCommandLists,
            >(hook_cqecl.trampoline()),
        });

        Self([hook_present, hook_resize_buffers, hook_cqecl])
    }
}

impl Hooks for Dx12Hooks {
    fn from_render_loop<T>(t: T) -> Box<Self>
    where
        Self: Sized,
        T: EguiRenderLoop + Send + Sync + 'static,
    {
        Box::new(unsafe { Self::new(t) })
    }

    fn hooks(&self) -> &[MhHook] {
        &self.0
    }

    unsafe fn unhook(&mut self) {
        TRAMPOLINES.take();
        PIPELINE.take();
        RENDER_LOOP.take();
        *INITIALIZATION_CONTEXT.lock() = InitializationContext::Empty
    }
}
