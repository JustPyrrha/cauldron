use crate::util;
use crate::util::Fence;
use egui::{Color32, ImageData, TextureId, TexturesDelta};
use log::{error, warn};
use std::collections::HashMap;
use std::mem::ManuallyDrop;
use std::ptr;
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::core::{Error, HRESULT, Interface, Result, w};

pub struct Texture {
    pub resource: ID3D12Resource,
    pub gpu_desc: D3D12_GPU_DESCRIPTOR_HANDLE,
    pub width: u32,
    pub height: u32,
}

pub struct TextureHeap {
    pub device: ID3D12Device,
    pub srv_staging_heap: ID3D12DescriptorHeap,
    pub srv_heap: ID3D12DescriptorHeap,
    pub textures: HashMap<TextureId, Texture>,
    pub command_queue: ID3D12CommandQueue,
    pub command_allocator: ID3D12CommandAllocator,
    pub command_list: ID3D12GraphicsCommandList,
    pub fence: Fence,
}

impl TextureHeap {
    pub fn new(
        device: &ID3D12Device,
        srv_heap: ID3D12DescriptorHeap,
    ) -> windows::core::Result<Self> {
        let command_queue = unsafe {
            device.CreateCommandQueue(&D3D12_COMMAND_QUEUE_DESC {
                Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
                Priority: 0,
                Flags: D3D12_COMMAND_QUEUE_FLAG_NONE,
                NodeMask: 0,
            })
        }?;

        let command_allocator: ID3D12CommandAllocator =
            unsafe { device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT) }?;
        let command_list: ID3D12GraphicsCommandList = unsafe {
            device.CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, &command_allocator, None)
        }?;

        unsafe {
            command_list.Close()?;
            command_allocator.SetName(w!("sunwing::focus render engine command allocator"))?;
            command_list.SetName(w!("sunwing::focus render engine command list"))?;
        }

        let srv_staging_heap: ID3D12DescriptorHeap = unsafe {
            device.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
                Type: D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
                NumDescriptors: 8,
                Flags: D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
                NodeMask: 0,
            })
        }?;

        let fence = Fence::new(&device)?;

        Ok(Self {
            device: device.clone(),
            srv_heap,
            srv_staging_heap,
            textures: HashMap::new(),
            command_queue,
            command_allocator,
            command_list,
            fence,
        })
    }

    unsafe fn resize_heap(&mut self) -> Result<()> {
        unsafe {
            let mut desc = self.srv_heap.GetDesc();
            let mut desc_staging = self.srv_staging_heap.GetDesc();
            let old_num_descriptors = desc.NumDescriptors;

            if old_num_descriptors <= self.textures.len() as _ {
                desc.NumDescriptors *= 2;
                desc_staging.NumDescriptors = desc.NumDescriptors;

                let srv_heap: ID3D12DescriptorHeap = self.device.CreateDescriptorHeap(&desc)?;
                let srv_staging_heap: ID3D12DescriptorHeap =
                    self.device.CreateDescriptorHeap(&desc_staging)?;
                self.device.CopyDescriptorsSimple(
                    old_num_descriptors,
                    srv_staging_heap.GetCPUDescriptorHandleForHeapStart(),
                    self.srv_heap.GetCPUDescriptorHandleForHeapStart(),
                    D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
                );
                self.device.CopyDescriptorsSimple(
                    old_num_descriptors,
                    srv_heap.GetCPUDescriptorHandleForHeapStart(),
                    self.srv_heap.GetCPUDescriptorHandleForHeapStart(),
                    D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
                );
                self.srv_heap = srv_heap;
                self.srv_staging_heap = srv_staging_heap;
            }

            let gpu_heap_start = self.srv_heap.GetGPUDescriptorHandleForHeapStart();
            let heap_inc_size = self
                .device
                .GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV);
            self.textures
                .iter_mut()
                .enumerate()
                .for_each(|(idx, (_tid, texture))| {
                    texture.gpu_desc = D3D12_GPU_DESCRIPTOR_HANDLE {
                        ptr: gpu_heap_start.ptr + (idx as u32 * heap_inc_size) as u64,
                    }
                });

            Ok(())
        }
    }

    unsafe fn create_texture(
        &mut self,
        tid: TextureId,
        width: u32,
        height: u32,
    ) -> Result<TextureId> {
        unsafe {
            self.resize_heap()?;

            let cpu_heap_stg_start = self.srv_staging_heap.GetCPUDescriptorHandleForHeapStart();
            let cpu_heap_start = self.srv_heap.GetCPUDescriptorHandleForHeapStart();
            let gpu_heap_start = self.srv_heap.GetGPUDescriptorHandleForHeapStart();
            let heap_inc_size = self
                .device
                .GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV);

            let texture_index = self.textures.len() as u32;

            let cpu_desc_stg = D3D12_CPU_DESCRIPTOR_HANDLE {
                ptr: cpu_heap_stg_start.ptr + (texture_index * heap_inc_size) as usize,
            };

            let cpu_desc = D3D12_CPU_DESCRIPTOR_HANDLE {
                ptr: cpu_heap_start.ptr + (texture_index * heap_inc_size) as usize,
            };

            let gpu_desc = D3D12_GPU_DESCRIPTOR_HANDLE {
                ptr: gpu_heap_start.ptr + (texture_index * heap_inc_size) as u64,
            };

            let texture: ID3D12Resource = util::try_out_ptr(|v| {
                self.device.CreateCommittedResource(
                    &D3D12_HEAP_PROPERTIES {
                        Type: D3D12_HEAP_TYPE_DEFAULT,
                        CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
                        MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
                        CreationNodeMask: Default::default(),
                        VisibleNodeMask: Default::default(),
                    },
                    D3D12_HEAP_FLAG_NONE,
                    &D3D12_RESOURCE_DESC {
                        Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
                        Alignment: 0,
                        Width: width as _,
                        Height: height as _,
                        DepthOrArraySize: 1,
                        MipLevels: 1,
                        Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                        SampleDesc: DXGI_SAMPLE_DESC {
                            Count: 1,
                            Quality: 0,
                        },
                        Layout: D3D12_TEXTURE_LAYOUT_UNKNOWN,
                        Flags: D3D12_RESOURCE_FLAG_NONE,
                    },
                    D3D12_RESOURCE_STATE_COPY_DEST,
                    None,
                    v,
                )
            })?;

            self.device.CreateShaderResourceView(
                &texture,
                Some(&D3D12_SHADER_RESOURCE_VIEW_DESC {
                    Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                    ViewDimension: D3D12_SRV_DIMENSION_TEXTURE2D,
                    Shader4ComponentMapping: D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING,
                    Anonymous: D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
                        Texture2D: D3D12_TEX2D_SRV {
                            MostDetailedMip: 0,
                            MipLevels: 1,
                            PlaneSlice: Default::default(),
                            ResourceMinLODClamp: Default::default(),
                        },
                    },
                }),
                cpu_desc_stg,
            );

            self.device.CopyDescriptorsSimple(
                1,
                cpu_desc,
                cpu_desc_stg,
                D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
            );

            self.textures.insert(
                tid,
                Texture {
                    resource: texture,
                    gpu_desc,
                    width,
                    height,
                },
            );

            Ok(tid)
        }
    }

    unsafe fn upload_texture(
        &mut self,
        texture_id: TextureId,
        data: &[u8],
        width: u32,
        height: u32,
        is_partial: bool,
        copy_x: u32,
        copy_y: u32,
    ) -> Result<()> {
        unsafe {
            let texture = &self.textures[&texture_id];
            if (texture.width != width || texture.height != height as _) && !is_partial {
                error!(
                    "image size {width}x{height} does not match expected {}x{}",
                    texture.width, texture.height
                );
                return Err(Error::from_hresult(HRESULT(-1)));
            }

            let upload_row_size = width * 4;
            let align = D3D12_TEXTURE_DATA_PITCH_ALIGNMENT;
            let upload_pitch = upload_row_size.div_ceil(align) * align; // 256 bytes aligned
            let upload_size = height * upload_pitch;

            let upload_buffer: ID3D12Resource = util::try_out_ptr(|v| {
                self.device.CreateCommittedResource(
                    &D3D12_HEAP_PROPERTIES {
                        Type: D3D12_HEAP_TYPE_UPLOAD,
                        CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
                        MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
                        CreationNodeMask: Default::default(),
                        VisibleNodeMask: Default::default(),
                    },
                    D3D12_HEAP_FLAG_NONE,
                    &D3D12_RESOURCE_DESC {
                        Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
                        Alignment: 0,
                        Width: upload_size as _,
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
            })?;

            let mut upload_buffer_ptr = ptr::null_mut();
            upload_buffer.Map(0, None, Some(&mut upload_buffer_ptr))?;
            if upload_row_size == upload_pitch {
                ptr::copy_nonoverlapping(data.as_ptr(), upload_buffer_ptr as *mut u8, data.len());
            } else {
                for y in 0..height {
                    let src = data.as_ptr().add((y * upload_row_size) as usize);
                    let dst = (upload_buffer_ptr as *mut u8).add((y * upload_row_size) as usize);
                    ptr::copy_nonoverlapping(src, dst, upload_row_size as usize);
                }
            }
            upload_buffer.Unmap(0, None);

            self.command_allocator.Reset()?;
            self.command_list.Reset(&self.command_allocator, None)?;

            let dst_location = D3D12_TEXTURE_COPY_LOCATION {
                pResource: ManuallyDrop::new(Some(texture.resource.clone())),
                Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
                Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                    SubresourceIndex: 0,
                },
            };

            let src_location = D3D12_TEXTURE_COPY_LOCATION {
                pResource: ManuallyDrop::new(Some(upload_buffer.clone())),
                Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
                Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                    PlacedFootprint: D3D12_PLACED_SUBRESOURCE_FOOTPRINT {
                        Offset: 0,
                        Footprint: D3D12_SUBRESOURCE_FOOTPRINT {
                            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                            Width: width,
                            Height: height,
                            Depth: 1,
                            RowPitch: upload_pitch,
                        },
                    },
                },
            };

            self.command_list.CopyTextureRegion(
                &dst_location,
                copy_x,
                copy_y,
                0,
                &src_location,
                None,
            );
            let barriers = [util::create_barrier(
                &texture.resource,
                D3D12_RESOURCE_STATE_COPY_DEST,
                D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE,
            )];

            self.command_list.ResourceBarrier(&barriers);
            self.command_list.Close()?;
            self.command_queue
                .ExecuteCommandLists(&[Some(self.command_list.cast()?)]);
            self.command_queue
                .Signal(self.fence.fence(), self.fence.value())?;
            self.fence.wait()?;
            self.fence.incr();

            barriers.into_iter().for_each(util::drop_barrier);

            let _ = ManuallyDrop::into_inner(dst_location.pResource);

            Ok(())
        }
    }

    pub unsafe fn update(&mut self, delta: TexturesDelta) -> Result<()> {
        unsafe {
            for (tid, delta) in delta.set {
                match delta.image {
                    ImageData::Color(ref ci) => {
                        if delta.is_whole() {
                            self.create_texture(
                                tid,
                                delta.image.width() as _,
                                delta.image.height() as _,
                            )?;
                            let data = ci
                                .pixels
                                .clone()
                                .iter()
                                .map(|c| c.to_array())
                                .flatten()
                                .collect::<Vec<u8>>();
                            self.upload_texture(
                                tid,
                                data.as_slice(),
                                delta.image.width() as _,
                                delta.image.height() as _,
                                false,
                                0,
                                0,
                            )?;
                        } else if let Some(_) = self.textures.get(&tid) {
                            warn!(
                                "egui is trying to modify a color texture {tid:?}, this *should* only happen for fonts and will be ignored."
                            );
                        } else {
                            warn!(
                                "egui is trying to update a non-existent texture {tid:?}. ignoring."
                            );
                        }
                    }
                    ImageData::Font(ref fi) => {
                        let new = fi
                            .pixels
                            .clone()
                            .iter()
                            .map(|a| {
                                Color32::from_rgba_premultiplied(255, 255, 255, (a * 255.) as u8)
                            })
                            .map(|c| c.to_array())
                            .flatten()
                            .collect::<Vec<u8>>();

                        if delta.is_whole() {
                            self.create_texture(
                                tid,
                                delta.image.width() as _,
                                delta.image.height() as _,
                            )?;
                            self.upload_texture(
                                tid,
                                new.as_slice(),
                                delta.image.width() as _,
                                delta.image.height() as _,
                                false,
                                0,
                                0,
                            )?;
                        } else if let Some(_) = self.textures.get_mut(&tid) {
                            self.upload_texture(
                                tid,
                                new.as_slice(),
                                delta.image.width() as _,
                                delta.image.height() as _,
                                true,
                                delta.pos.unwrap()[0] as _,
                                delta.pos.unwrap()[1] as _,
                            )?
                        } else {
                            warn!(
                                "egui is trying to update a non-existent texture {tid:?}. ignoring."
                            );
                        }
                    }
                }
            }

            Ok(())
        }
    }
}
