use log::{debug, error};
use std::ffi::c_void;
use std::fmt::Display;
use std::mem::ManuallyDrop;
use std::sync::atomic::{AtomicU64, Ordering};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Graphics::Direct3D::ID3DBlob;
use windows::Win32::Graphics::Direct3D12::{
    D3D12GetDebugInterface, ID3D12Debug6, ID3D12Device, ID3D12Fence, ID3D12Resource,
    D3D12_FENCE_FLAG_NONE, D3D12_RESOURCE_BARRIER, D3D12_RESOURCE_BARRIER_0,
    D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES, D3D12_RESOURCE_BARRIER_FLAG_NONE,
    D3D12_RESOURCE_BARRIER_TYPE_TRANSITION, D3D12_RESOURCE_STATES,
    D3D12_RESOURCE_TRANSITION_BARRIER,
};
use windows::Win32::Graphics::Dxgi::{
    DXGIGetDebugInterface1, IDXGIInfoQueue, DXGI_DEBUG_ALL, DXGI_INFO_QUEUE_MESSAGE,
};
use windows::Win32::System::Memory::{
    VirtualQuery, MEMORY_BASIC_INFORMATION, PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE,
    PAGE_PROTECTION_FLAGS, PAGE_READONLY, PAGE_READWRITE,
};
use windows::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO};
use windows::Win32::System::Threading::{CreateEventExW, WaitForSingleObjectEx, CREATE_EVENT};

/// Wrapper around [`windows::Win32::Graphics::Direct3D12::ID3D12Fence`].
pub struct Fence {
    fence: ID3D12Fence,
    value: AtomicU64,
    event: HANDLE,
}

impl Fence {
    pub fn new(device: &ID3D12Device) -> windows::core::Result<Self> {
        let fence = unsafe { device.CreateFence(0, D3D12_FENCE_FLAG_NONE) }?;
        let value = AtomicU64::new(0);
        let event = unsafe { CreateEventExW(None, None, CREATE_EVENT(0), 0x1f0003) }?;

        Ok(Fence {
            fence,
            value,
            event,
        })
    }
    pub fn fence(&self) -> &ID3D12Fence {
        &self.fence
    }

    pub fn value(&self) -> u64 {
        self.value.load(Ordering::SeqCst)
    }
    pub fn incr(&self) {
        self.value.fetch_add(1, Ordering::SeqCst);
    }


    pub fn wait(&self) -> windows::core::Result<()> {
        let value = self.value();
        unsafe {
            if self.fence.GetCompletedValue() < value {
                self.fence.SetEventOnCompletion(value, self.event)?;
                WaitForSingleObjectEx(self.event, u32::MAX, false);
            }
        }

        Ok(())
    }
}

pub fn try_out_ptr<T, F, E, O>(mut f: F) -> Result<T, E>
where
    F: FnMut(&mut Option<T>) -> Result<O, E>,
{
    let mut t: Option<T> = None;
    match f(&mut t) {
        Ok(_) => Ok(t.unwrap()),
        Err(e) => Err(e),
    }
}

pub fn create_barrier(
    resource: &ID3D12Resource,
    before: D3D12_RESOURCE_STATES,
    after: D3D12_RESOURCE_STATES,
) -> D3D12_RESOURCE_BARRIER {
    D3D12_RESOURCE_BARRIER {
        Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
        Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
        Anonymous: D3D12_RESOURCE_BARRIER_0 {
            Transition: ManuallyDrop::new(D3D12_RESOURCE_TRANSITION_BARRIER {
                pResource: ManuallyDrop::new(Some(resource.clone())),
                Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                StateBefore: before,
                StateAfter: after,
            }),
        },
    }
}

pub fn drop_barrier(barrier: D3D12_RESOURCE_BARRIER) {
    let transition = ManuallyDrop::into_inner(unsafe { barrier.Anonymous.Transition });
    let _ = ManuallyDrop::into_inner(transition.pResource);
}

pub fn try_out_err_blob<T1, T2, F, E, O>(mut f: F) -> Result<T1, (E, T2)>
where
    F: FnMut(&mut Option<T1>, &mut Option<T2>) -> Result<O, E>,
{
    let mut t1: Option<T1> = None;
    let mut t2: Option<T2> = None;
    match f(&mut t1, &mut t2) {
        Ok(_) => Ok(t1.unwrap()),
        Err(e) => Err((e, t2.unwrap())),
    }
}

pub fn print_error_blob<D: Display, E>(msg: D) -> impl Fn((E, ID3DBlob)) -> E {
    move |(e, err_blob): (E, ID3DBlob)| {
        let buf_ptr = unsafe { err_blob.GetBufferPointer() } as *mut u8;
        let buf_size = unsafe { err_blob.GetBufferSize() };
        let s = unsafe { String::from_raw_parts(buf_ptr, buf_size, buf_size + 1) };
        error!("{msg}: {s}");
        e
    }
}

pub fn enable_debug_interface(enable_gpu_validation: bool) {
    let debug_interface: Result<ID3D12Debug6, _> =
        try_out_ptr(|v| unsafe { D3D12GetDebugInterface(v) });

    match debug_interface {
        Ok(debug_interface) => unsafe {
            debug_interface.EnableDebugLayer();
            if enable_gpu_validation {
                debug_interface.SetEnableGPUBasedValidation(true);
            }
        },
        Err(e) => {
            error!("Could not create debug interface: {e:?}")
        }
    }
}

pub fn print_dxgi_debug_messages() {
    let Ok(diq): Result<IDXGIInfoQueue, _> = (unsafe { DXGIGetDebugInterface1(0) }) else {
        return;
    };

    let n = unsafe { diq.GetNumStoredMessages(DXGI_DEBUG_ALL) };
    for i in 0..n {
        let mut msg_len: usize = 0;
        unsafe {
            diq.GetMessage(DXGI_DEBUG_ALL, i, None, &mut msg_len as _)
                .unwrap()
        };
        let diqm = vec![0u8; msg_len];
        let pdiqm = diqm.as_ptr() as *mut DXGI_INFO_QUEUE_MESSAGE;
        unsafe {
            diq.GetMessage(DXGI_DEBUG_ALL, i, Some(pdiqm), &mut msg_len as _)
                .unwrap()
        };
        let diqm = unsafe { pdiqm.as_ref().unwrap() };
        debug!(
            "[DIQ] {}",
            String::from_utf8_lossy(unsafe {
                std::slice::from_raw_parts(diqm.pDescription, diqm.DescriptionByteLength - 1)
            })
        );
    }
    unsafe { diq.ClearStoredMessages(DXGI_DEBUG_ALL) };
}

pub unsafe fn readable_region<T>(ptr: *const T, limit: usize) -> &'static [T] {
    //Check if the page pointed to by `ptr.rs` is readable.
    unsafe fn is_readable(
        ptr: *const c_void,
        memory_basic_info: &mut MEMORY_BASIC_INFORMATION,
    ) -> bool {
        // If the page protection has any of these flags set, we can read from it
        const PAGE_READABLE: PAGE_PROTECTION_FLAGS = PAGE_PROTECTION_FLAGS(
            PAGE_READONLY.0 | PAGE_READWRITE.0 | PAGE_EXECUTE_READ.0 | PAGE_EXECUTE_READWRITE.0,
        );

        (unsafe {
            VirtualQuery(
                Some(ptr),
                memory_basic_info,
                size_of::<MEMORY_BASIC_INFORMATION>(),
            )
        } != 0)
            && (memory_basic_info.Protect & PAGE_READABLE).0 != 0
    }

    // This is probably 0x1000 (4096) bytes
    let page_size_bytes = {
        let mut system_info = SYSTEM_INFO::default();
        unsafe { GetSystemInfo(&mut system_info) };
        system_info.dwPageSize as usize
    };
    let page_align_mask = page_size_bytes - 1;

    // Calculate the starting address of the first and last pages that need to be
    // readable in order to read `limit` elements of type `T` from `ptr.rs`
    let first_page_addr = (ptr as usize) & !page_align_mask;
    let last_page_addr = (ptr as usize + (limit * size_of::<T>()) - 1) & !page_align_mask;

    let mut memory_basic_info = MEMORY_BASIC_INFORMATION::default();
    for page_addr in (first_page_addr..=last_page_addr).step_by(page_size_bytes) {
        if unsafe { is_readable(page_addr as _, &mut memory_basic_info) } {
            continue;
        }

        // If this page is not readable, we can read from `ptr.rs`
        // up to (not including) the start of this page
        //
        // Note: `page_addr` can be less than `ptr.rs` if `ptr.rs` is not page-aligned
        let num_readable = page_addr.saturating_sub(ptr as usize) / size_of::<T>();

        // SAFETY:
        // - `ptr.rs` is a valid pointer to `limit` elements of type `T`
        // - `num_readable` is always less than or equal to `limit`
        return std::slice::from_raw_parts(ptr, num_readable);
    }

    // SAFETY:
    // - `ptr.rs` is a valid pointer to `limit` elements of type `T` and is properly
    //   aligned
    std::slice::from_raw_parts(ptr, limit)
}
