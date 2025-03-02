use crate::egui_d3d12::util::try_out_ptr;
use std::{mem, ptr};
use windows::Win32::Graphics::Direct3D12::{
    D3D12_CPU_PAGE_PROPERTY_UNKNOWN, D3D12_HEAP_FLAG_NONE, D3D12_HEAP_PROPERTIES,
    D3D12_HEAP_TYPE_UPLOAD, D3D12_MEMORY_POOL_UNKNOWN, D3D12_RESOURCE_DESC,
    D3D12_RESOURCE_DIMENSION_BUFFER, D3D12_RESOURCE_FLAG_NONE, D3D12_RESOURCE_STATE_GENERIC_READ,
    D3D12_TEXTURE_LAYOUT_ROW_MAJOR, ID3D12Device, ID3D12Resource,
};
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_UNKNOWN, DXGI_SAMPLE_DESC};

pub struct Buffer<T: Sized> {
    pub resource: ID3D12Resource,
    pub resource_capacity: usize,
    pub data: Vec<T>,
}

impl<T> Buffer<T> {
    pub fn new(device: &ID3D12Device, resource_capacity: usize) -> windows::core::Result<Self> {
        let resource = Self::create_resource(device, resource_capacity)?;
        let data = Vec::with_capacity(resource_capacity);

        Ok(Self {
            resource,
            resource_capacity,
            data,
        })
    }

    pub fn create_resource(
        device: &ID3D12Device,
        resource_capacity: usize,
    ) -> windows::core::Result<ID3D12Resource> {
        try_out_ptr(|v| unsafe {
            device.CreateCommittedResource(
                &D3D12_HEAP_PROPERTIES {
                    Type: D3D12_HEAP_TYPE_UPLOAD,
                    CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
                    MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
                    CreationNodeMask: 0,
                    VisibleNodeMask: 0,
                },
                D3D12_HEAP_FLAG_NONE,
                &D3D12_RESOURCE_DESC {
                    Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
                    Alignment: 0,
                    Width: (resource_capacity * mem::size_of::<T>()) as u64,
                    Height: 1,
                    DepthOrArraySize: 1,
                    MipLevels: 1,
                    Format: DXGI_FORMAT_UNKNOWN,
                    SampleDesc: DXGI_SAMPLE_DESC {
                        Count: 1,
                        Quality: 0,
                    },
                    Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
                    Flags: D3D12_RESOURCE_FLAG_NONE,
                },
                D3D12_RESOURCE_STATE_GENERIC_READ,
                None,
                v,
            )
        })
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn extend<I: IntoIterator<Item = T>>(&mut self, it: I) {
        self.data.extend(it)
    }

    pub fn upload(&mut self, device: &ID3D12Device) -> windows::core::Result<()> {
        let capacity = self.data.capacity();
        if capacity > self.resource_capacity {
            drop(mem::replace(
                &mut self.resource,
                Self::create_resource(device, capacity)?,
            ));
            self.resource_capacity = capacity;
        }

        unsafe {
            let mut resource_ptr = ptr::null_mut();
            self.resource.Map(0, None, Some(&mut resource_ptr))?;
            ptr::copy_nonoverlapping(self.data.as_ptr(), resource_ptr as *mut T, self.data.len());
            self.resource.Unmap(0, None);
        }

        Ok(())
    }
}
