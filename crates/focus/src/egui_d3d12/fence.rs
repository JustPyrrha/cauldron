use std::sync::atomic::{AtomicU64, Ordering};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Graphics::Direct3D12::{D3D12_FENCE_FLAG_NONE, ID3D12Device, ID3D12Fence};
use windows::Win32::System::Threading::{CREATE_EVENT, CreateEventExW, WaitForSingleObjectEx};

pub struct Fence {
    fence: ID3D12Fence,
    value: AtomicU64,
    event: usize,
}

impl Fence {
    pub fn new(device: &ID3D12Device) -> windows::core::Result<Self> {
        let fence = unsafe { device.CreateFence(0, D3D12_FENCE_FLAG_NONE) }?;
        let value = AtomicU64::new(0);
        let event = unsafe { CreateEventExW(None, None, CREATE_EVENT(0), 0x1f0003) }?;

        Ok(Fence {
            fence,
            value,
            event: event.0 as usize,
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
                self.fence
                    .SetEventOnCompletion(value, HANDLE(self.event as *mut _))?;
                WaitForSingleObjectEx(HANDLE(self.event as *mut _), u32::MAX, false);
            }
        }

        Ok(())
    }
}
