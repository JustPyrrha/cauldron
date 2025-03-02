use crate::{assert_offset, assert_size, gen_with_vtbl};
use windows::Win32::Graphics::Direct3D12::ID3D12Device;

gen_with_vtbl!(
    NxD3D12Driver,

    fn fn_destructor();

    pub pad_8: [u8;0x30],
    pub device: *mut ID3D12Device,
    pub pad_40: [u8;0xD8],
);

assert_size!(NxD3D12Driver, 0x118);
assert_offset!(NxD3D12Driver, device, 0x38);
