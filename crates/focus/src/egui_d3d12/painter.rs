use crate::egui_d3d12::buffer::Buffer;
use crate::egui_d3d12::fence::Fence;
use crate::egui_d3d12::texture::TextureHeap;
use crate::egui_d3d12::util::{
    create_barrier, drop_barrier, print_error_blob, try_out_err_blob, try_out_ptr,
};
use egui::epaint::{ClippedShape, Primitive, Vertex};
use egui::{
    ClippedPrimitive, Context, FullOutput, PlatformOutput, Pos2, Rgba, TexturesDelta,
    ViewportIdMap, ViewportOutput,
};
use std::fmt::Formatter;
use std::mem::{ManuallyDrop, offset_of};
use std::slice;
use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Direct3D::Fxc::D3DCompile;
use windows::Win32::Graphics::Direct3D::{
    D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST, ID3DBlob, ID3DInclude,
};
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_FORMAT_D32_FLOAT, DXGI_FORMAT_R8G8B8A8_UNORM_SRGB, DXGI_FORMAT_R32_UINT,
    DXGI_FORMAT_R32G32_FLOAT, DXGI_FORMAT_R32G32B32A32_FLOAT, DXGI_SAMPLE_DESC,
};
use windows::core::{Interface, Result, s, w};

#[derive(Debug)]
pub struct PainterError(String);

impl std::error::Error for PainterError {}

impl std::fmt::Display for PainterError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "D3D12: {}", self.0)
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct VertexData {
    pos: Pos2,
    uv: Pos2,
    color: Rgba,
}

struct MeshData {
    vtx: Vec<VertexData>,
    idx: Vec<u32>,
    tex: egui::TextureId,
    clip_rect: egui::Rect,
}

pub struct RendererOutput {
    pub textures_delta: TexturesDelta,
    pub shapes: Vec<ClippedShape>,
    pub pixels_per_point: f32,
}

pub fn split_output(
    full_output: FullOutput,
) -> (
    RendererOutput,
    PlatformOutput,
    ViewportIdMap<ViewportOutput>,
) {
    (
        RendererOutput {
            textures_delta: full_output.textures_delta,
            shapes: full_output.shapes,
            pixels_per_point: full_output.pixels_per_point,
        },
        full_output.platform_output,
        full_output.viewport_output,
    )
}

pub struct Painter {
    pub device: ID3D12Device,

    pub command_queue: ID3D12CommandQueue,
    pub command_allocator: ID3D12CommandAllocator,
    pub command_list: ID3D12GraphicsCommandList,

    #[allow(unused)]
    pub rtv_heap: ID3D12DescriptorHeap,
    pub rtv_heap_start: D3D12_CPU_DESCRIPTOR_HANDLE,
    pub texture_heap: TextureHeap,

    pub root_signature: ID3D12RootSignature,
    pub pipeline_state: ID3D12PipelineState,

    pub vertex_buf: Buffer<VertexData>,
    pub index_buf: Buffer<u32>,

    pub fence: Fence,
}

impl Painter {
    pub fn new(command_queue: &ID3D12CommandQueue) -> Result<Self> {
        let command_queue = command_queue.clone();
        let device: ID3D12Device =
            try_out_ptr(|device| unsafe { command_queue.GetDevice(device) })?;
        let command_allocator: ID3D12CommandAllocator =
            unsafe { device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT)? };
        let command_list: ID3D12GraphicsCommandList = unsafe {
            device.CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, &command_allocator, None)?
        };
        unsafe { command_list.Close()? };

        unsafe {
            command_allocator.SetName(w!("cauldron::focus::egui_d3d12 allocator"))?;
            command_list.SetName(w!("cauldron::focus::egui_d3d12 list"))?;
        }

        let rtv_heap: ID3D12DescriptorHeap = unsafe {
            device.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
                Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
                NumDescriptors: 1,
                Flags: D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
                NodeMask: 1,
            })?
        };
        let srv_heap: ID3D12DescriptorHeap = unsafe {
            device.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
                Type: D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
                NumDescriptors: 8,
                Flags: D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE,
                NodeMask: 0,
            })?
        };
        let texture_heap = TextureHeap::new(&device, srv_heap)?;
        let rtv_heap_start = unsafe { rtv_heap.GetCPUDescriptorHandleForHeapStart() };

        let parameters = [
            D3D12_ROOT_PARAMETER {
                ParameterType: D3D12_ROOT_PARAMETER_TYPE_32BIT_CONSTANTS,
                Anonymous: D3D12_ROOT_PARAMETER_0 {
                    Constants: D3D12_ROOT_CONSTANTS {
                        ShaderRegister: 0,
                        RegisterSpace: 0,
                        Num32BitValues: 16,
                    },
                },
                ShaderVisibility: D3D12_SHADER_VISIBILITY_VERTEX,
            },
            D3D12_ROOT_PARAMETER {
                ParameterType: D3D12_ROOT_PARAMETER_TYPE_DESCRIPTOR_TABLE,
                Anonymous: D3D12_ROOT_PARAMETER_0 {
                    DescriptorTable: D3D12_ROOT_DESCRIPTOR_TABLE {
                        NumDescriptorRanges: 1,
                        pDescriptorRanges: &D3D12_DESCRIPTOR_RANGE {
                            RangeType: D3D12_DESCRIPTOR_RANGE_TYPE_SRV,
                            NumDescriptors: 1,
                            BaseShaderRegister: 0,
                            RegisterSpace: 0,
                            OffsetInDescriptorsFromTableStart: 0,
                        },
                    },
                },
                ShaderVisibility: D3D12_SHADER_VISIBILITY_PIXEL,
            },
        ];

        let root_signature_desc = D3D12_ROOT_SIGNATURE_DESC {
            NumParameters: 2,
            pParameters: parameters.as_ptr(),
            NumStaticSamplers: 1,
            pStaticSamplers: &D3D12_STATIC_SAMPLER_DESC {
                Filter: D3D12_FILTER_MIN_MAG_MIP_LINEAR,
                AddressU: D3D12_TEXTURE_ADDRESS_MODE_BORDER,
                AddressV: D3D12_TEXTURE_ADDRESS_MODE_BORDER,
                AddressW: D3D12_TEXTURE_ADDRESS_MODE_BORDER,
                MipLODBias: 0f32,
                MaxAnisotropy: 0,
                ComparisonFunc: D3D12_COMPARISON_FUNC_ALWAYS,
                BorderColor: D3D12_STATIC_BORDER_COLOR_OPAQUE_WHITE,
                MinLOD: 0f32,
                MaxLOD: 0f32,
                ShaderRegister: 0,
                RegisterSpace: 0,
                ShaderVisibility: D3D12_SHADER_VISIBILITY_PIXEL,
            },
            Flags: D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT
                | D3D12_ROOT_SIGNATURE_FLAG_DENY_HULL_SHADER_ROOT_ACCESS
                | D3D12_ROOT_SIGNATURE_FLAG_DENY_DOMAIN_SHADER_ROOT_ACCESS
                | D3D12_ROOT_SIGNATURE_FLAG_DENY_GEOMETRY_SHADER_ROOT_ACCESS,
        };

        let blob: ID3DBlob = try_out_err_blob(|v, err_blob| unsafe {
            D3D12SerializeRootSignature(
                &root_signature_desc,
                D3D_ROOT_SIGNATURE_VERSION_1_0,
                v,
                Some(err_blob),
            )
        })
        .map_err(print_error_blob("Serializing root signature"))
        .expect("D3D12SerializeRootSignature");

        let root_signature: ID3D12RootSignature = unsafe {
            device.CreateRootSignature(
                0,
                slice::from_raw_parts(blob.GetBufferPointer() as *const u8, blob.GetBufferSize()),
            )?
        };

        const VS: &str = r#"
    struct vs_in {
      float2 position : POSITION;
      float2 uv : TEXCOORD;
      float4 color : COLOR;
    };

    struct vs_out {
      float4 clip : SV_POSITION;
      float2 uv : TEXCOORD;
      float4 color : COLOR;
    };

    vs_out vs_main(vs_in input) {
      vs_out output;
      output.clip = float4(input.position, 0.0, 1.0);
      output.uv = input.uv;
      output.color = input.color;

      return output;
    }"#;

        const PS: &str = r#"
    struct vs_out {
      float4 clip : SV_POSITION;
      float2 uv : TEXCOORD;
      float4 color : COLOR;
    };
    sampler sampler0;
    Texture2D texture0;

    float4 ps_main(vs_out input) : SV_TARGET {
      return pow(input.color, 1.0 / 2.2) * texture0.Sample(sampler0, input.uv);
    }"#;

        let vtx_shader: ID3DBlob = try_out_err_blob(|v, err_blob| unsafe {
            D3DCompile(
                VS.as_ptr() as _,
                VS.len(),
                None,
                None,
                None::<&ID3DInclude>,
                s!("vs_main\0"),
                s!("vs_5_0\0"),
                0,
                0,
                v,
                Some(err_blob),
            )
        })
        .map_err(print_error_blob("Compiling vertex shader"))
        .expect("D3DCompile");

        let pix_shader = try_out_err_blob(|v, err_blob| unsafe {
            D3DCompile(
                PS.as_ptr() as _,
                PS.len(),
                None,
                None,
                None::<&ID3DInclude>,
                s!("ps_main\0"),
                s!("ps_5_0\0"),
                0,
                0,
                v,
                Some(err_blob),
            )
        })
        .map_err(print_error_blob("Compiling pixel shader"))
        .expect("D3DCompile");

        let input_elements = [
            D3D12_INPUT_ELEMENT_DESC {
                SemanticName: s!("POSITION"),
                SemanticIndex: 0,
                Format: DXGI_FORMAT_R32G32_FLOAT,
                InputSlot: 0,
                AlignedByteOffset: offset_of!(VertexData, pos) as u32,
                InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
                InstanceDataStepRate: 0,
            },
            D3D12_INPUT_ELEMENT_DESC {
                SemanticName: s!("TEXCOORD"),
                SemanticIndex: 0,
                Format: DXGI_FORMAT_R32G32_FLOAT,
                InputSlot: 0,
                AlignedByteOffset: offset_of!(VertexData, uv) as u32,
                InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
                InstanceDataStepRate: 0,
            },
            D3D12_INPUT_ELEMENT_DESC {
                SemanticName: s!("COLOR"),
                SemanticIndex: 0,
                Format: DXGI_FORMAT_R32G32B32A32_FLOAT,
                InputSlot: 0,
                AlignedByteOffset: offset_of!(VertexData, color) as u32,
                InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
                InstanceDataStepRate: 0,
            },
        ];

        let pso_desc = D3D12_GRAPHICS_PIPELINE_STATE_DESC {
            pRootSignature: ManuallyDrop::new(Some(root_signature.clone())),
            NodeMask: 1,
            PrimitiveTopologyType: D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
            SampleMask: u32::MAX,
            NumRenderTargets: 1,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Flags: D3D12_PIPELINE_STATE_FLAG_NONE,
            RTVFormats: [
                DXGI_FORMAT_R8G8B8A8_UNORM_SRGB,
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
            ],
            DSVFormat: DXGI_FORMAT_D32_FLOAT,
            VS: D3D12_SHADER_BYTECODE {
                pShaderBytecode: unsafe { vtx_shader.GetBufferPointer() },
                BytecodeLength: unsafe { vtx_shader.GetBufferSize() },
            },
            PS: D3D12_SHADER_BYTECODE {
                pShaderBytecode: unsafe { pix_shader.GetBufferPointer() },
                BytecodeLength: unsafe { pix_shader.GetBufferSize() },
            },
            InputLayout: D3D12_INPUT_LAYOUT_DESC {
                pInputElementDescs: input_elements.as_ptr(),
                NumElements: 3,
            },
            BlendState: D3D12_BLEND_DESC {
                AlphaToCoverageEnable: false.into(),
                IndependentBlendEnable: false.into(),
                RenderTarget: [
                    D3D12_RENDER_TARGET_BLEND_DESC {
                        BlendEnable: true.into(),
                        LogicOpEnable: false.into(),
                        SrcBlend: D3D12_BLEND_SRC_ALPHA,
                        DestBlend: D3D12_BLEND_INV_SRC_ALPHA,
                        BlendOp: D3D12_BLEND_OP_ADD,
                        SrcBlendAlpha: D3D12_BLEND_ONE,
                        DestBlendAlpha: D3D12_BLEND_INV_SRC_ALPHA,
                        BlendOpAlpha: D3D12_BLEND_OP_ADD,
                        LogicOp: Default::default(),
                        RenderTargetWriteMask: D3D12_COLOR_WRITE_ENABLE_ALL.0 as _,
                    },
                    Default::default(),
                    Default::default(),
                    Default::default(),
                    Default::default(),
                    Default::default(),
                    Default::default(),
                    Default::default(),
                ],
            },
            RasterizerState: D3D12_RASTERIZER_DESC {
                FillMode: D3D12_FILL_MODE_SOLID,
                CullMode: D3D12_CULL_MODE_NONE,
                FrontCounterClockwise: false.into(),
                DepthBias: D3D12_DEFAULT_DEPTH_BIAS,
                DepthBiasClamp: D3D12_DEFAULT_DEPTH_BIAS_CLAMP,
                SlopeScaledDepthBias: D3D12_DEFAULT_SLOPE_SCALED_DEPTH_BIAS,
                DepthClipEnable: true.into(),
                MultisampleEnable: false.into(),
                AntialiasedLineEnable: false.into(),
                ForcedSampleCount: 0,
                ConservativeRaster: D3D12_CONSERVATIVE_RASTERIZATION_MODE_OFF,
            },
            ..Default::default()
        };

        let pipeline_state = unsafe { device.CreateGraphicsPipelineState(&pso_desc)? };
        let _ = ManuallyDrop::into_inner(pso_desc.pRootSignature);

        let vertex_buf = Buffer::new(&device, 5000)?;
        let index_buf = Buffer::new(&device, 10000)?;

        let fence = Fence::new(&device)?;

        Ok(Self {
            device,
            command_queue,
            command_allocator,
            command_list,
            rtv_heap,
            rtv_heap_start,
            texture_heap,
            root_signature,
            pipeline_state,
            vertex_buf,
            index_buf,
            fence,
        })
    }

    pub fn render(
        &mut self,
        ctx: &Context,
        egui_output: RendererOutput,
        target: ID3D12Resource,
    ) -> Result<()> {
        unsafe {
            self.device
                .CreateRenderTargetView(&target, None, self.rtv_heap_start);

            self.command_allocator.Reset()?;
            self.command_list.Reset(&self.command_allocator, None)?;

            let present_to_rtv_barriers = [create_barrier(
                &target,
                D3D12_RESOURCE_STATE_PRESENT,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
            )];

            let rtv_to_present_barriers = [create_barrier(
                &target,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
                D3D12_RESOURCE_STATE_COMMON,
            )];

            self.command_list.ResourceBarrier(&present_to_rtv_barriers);
            self.command_list
                .OMSetRenderTargets(1, Some(&self.rtv_heap_start), false, None);
            self.command_list
                .SetDescriptorHeaps(&[Some(self.texture_heap.srv_heap.clone())]);

            self.render_egui(ctx, egui_output, &target)?;

            self.command_list.ResourceBarrier(&rtv_to_present_barriers);
            self.command_list.Close()?;
            self.command_queue
                .ExecuteCommandLists(&[Some(self.command_list.cast()?)]);
            self.command_queue
                .Signal(self.fence.fence(), self.fence.value())?;
            self.fence.wait()?;
            self.fence.incr();

            present_to_rtv_barriers.into_iter().for_each(drop_barrier);
            rtv_to_present_barriers.into_iter().for_each(drop_barrier)
        }

        Ok(())
    }

    unsafe fn render_egui(
        &mut self,
        ctx: &Context,
        egui_output: RendererOutput,
        target: &ID3D12Resource,
    ) -> Result<()> {
        unsafe {
            self.texture_heap.update(egui_output.textures_delta)?;
            if egui_output.shapes.is_empty() {
                return Ok(());
            }

            let frame_size = {
                let desc = target.GetDesc();
                (desc.Width as u32, desc.Height)
            };

            let zoom_factor = ctx.zoom_factor();
            let meshes = ctx
                .tessellate(egui_output.shapes, egui_output.pixels_per_point)
                .into_iter()
                .filter_map(
                    |ClippedPrimitive {
                         primitive,
                         clip_rect,
                     }| match primitive {
                        Primitive::Mesh(mesh) => Some((mesh, clip_rect)),
                        Primitive::Callback(..) => {
                            // todo: logging
                            // warn!("paint callbacks not yet supported.");
                            None
                        }
                    },
                )
                .filter_map(|(mesh, clip_rect)| {
                    if mesh.indices.is_empty() {
                        return None;
                    }
                    if mesh.indices.len() % 3 != 0 {
                        // todo: logging
                        // warn!("egui wants to draw an incomplete triangle. ignored.");
                        return None;
                    }
                    Some(MeshData {
                        vtx: mesh
                            .vertices
                            .into_iter()
                            .map(|Vertex { pos, uv, color }| VertexData {
                                pos: Pos2::new(
                                    pos.x * zoom_factor / frame_size.0 as f32 * 2.0 - 1.0,
                                    1.0 - pos.y * zoom_factor / frame_size.1 as f32 * 2.0,
                                ),
                                uv,
                                color: color.into(),
                            })
                            .collect(),
                        idx: mesh.indices,
                        tex: mesh.texture_id,
                        clip_rect: clip_rect /* * scale_factor */ * zoom_factor,
                    })
                })
                .collect::<Vec<MeshData>>();

            self.vertex_buf.clear();
            self.index_buf.clear();
            meshes
                .iter()
                .map(|data| (&data.vtx, &data.idx))
                .for_each(|(vtx, idx)| {
                    self.vertex_buf.extend(vtx.clone());
                    self.index_buf.extend(idx.clone());
                });
            self.vertex_buf
                .upload(&self.device)
                .expect("vertex buffer upload");
            self.index_buf
                .upload(&self.device)
                .expect("index buffer upload");

            self.command_list.RSSetViewports(&[D3D12_VIEWPORT {
                TopLeftX: 0f32,
                TopLeftY: 0f32,
                Width: frame_size.0 as f32,
                Height: frame_size.1 as f32,
                MinDepth: 0f32,
                MaxDepth: 1f32,
            }]);
            self.command_list.IASetVertexBuffers(
                0,
                Some(&[D3D12_VERTEX_BUFFER_VIEW {
                    BufferLocation: self.vertex_buf.resource.GetGPUVirtualAddress(),
                    SizeInBytes: (self.vertex_buf.data.len() * size_of::<VertexData>()) as _,
                    StrideInBytes: size_of::<VertexData>() as _,
                }]),
            );
            self.command_list
                .IASetIndexBuffer(Some(&D3D12_INDEX_BUFFER_VIEW {
                    BufferLocation: self.index_buf.resource.GetGPUVirtualAddress(),
                    SizeInBytes: (self.index_buf.data.len() * size_of::<u32>()) as _,
                    Format: DXGI_FORMAT_R32_UINT,
                }));

            self.command_list
                .IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
            self.command_list.SetPipelineState(&self.pipeline_state);
            self.command_list
                .SetGraphicsRootSignature(&self.root_signature);
            self.command_list.OMSetBlendFactor(Some(&[0f32; 4]));

            let mut vtx_offset = 0usize;
            let mut idx_offset = 0usize;

            for mesh in meshes {
                let texture_handle = self.texture_heap.textures[&mesh.tex].gpu_descriptor;
                self.command_list
                    .SetGraphicsRootDescriptorTable(1, texture_handle);
                self.command_list.RSSetScissorRects(&[RECT {
                    left: mesh.clip_rect.left() as _,
                    top: mesh.clip_rect.top() as _,
                    right: mesh.clip_rect.right() as _,
                    bottom: mesh.clip_rect.bottom() as _,
                }]);

                self.command_list.DrawIndexedInstanced(
                    mesh.idx.len() as _,
                    1,
                    idx_offset as _,
                    vtx_offset as _,
                    0,
                );
                vtx_offset += mesh.vtx.len();
                idx_offset += mesh.idx.len();
            }

            Ok(())
        }
    }
}
