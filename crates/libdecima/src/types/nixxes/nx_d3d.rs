use super::nx_d3d_driver::NxD3D12Driver;
use crate::{assert_offset, assert_size, gen_with_vtbl, impl_instance};
use std::os::raw::c_void;
use windows::Win32::Graphics::Direct3D12::ID3D12CommandQueue;

gen_with_vtbl!(
    NxD3DImpl,

    fn fn_destructor();
    fn fn_initialize(unk0: i32, unk1: u8) -> *mut c_void;
    fn fn_unk_10();
    fn fn_unk_18();
    fn fn_unk_20();
    fn fn_unk_28();
    fn fn_unk_30();
    fn fn_unk_38();
    fn fn_unk_40();
    fn fn_unk_48();
    fn fn_present(unk: *mut c_void);
    fn fn_unk_58();
    fn fn_unk_60();
    fn fn_unk_68();
    fn fn_unk_70();
    fn fn_unk_78();
    fn fn_unk_80();
    fn fn_unk_88();
    fn fn_unk_90();
    fn fn_unk_98();
    fn fn_unk_a0();
    fn fn_unk_a8();
    fn fn_unk_b0();
    fn fn_unk_b8();
    fn fn_unk_c0();
    fn fn_set_can_create_dx11_1_device();
    fn fn_get_back_buffer_count();
    fn fn_unk_d8();
    fn fn_unk_e0();
    fn fn_unk_e8();
    fn fn_unk_f0();
    fn fn_unk_f8();
    fn fn_unk_100();
    fn fn_get_command_queue(index: u32) -> *mut ID3D12CommandQueue;
    fn fn_unk_110();
    fn fn_unk_118();
    fn fn_unk_120();
    fn fn_unk_128();
    fn fn_unk_130();
    fn fn_unk_138();
    fn fn_unk_140();
    fn fn_unk_148();
    fn fn_unk_150();
    fn fn_unk_158();
    fn fn_unk_160();
    fn fn_unk_168();
    fn fn_unk_170();
    fn fn_unk_178();
    fn fn_unk_180();
    fn fn_can_create_dx11_1_device();
    fn fn_unk_190();
    fn fn_unk_198();
    fn fn_unk_1a0();
    fn fn_unk_1a8();
    fn fn_unk_1b0();

    pub pad_8: [u8;0x178],
    pub driver: *mut NxD3D12Driver,
    pub pad_38: [u8;0x78],
);

assert_size!(NxD3DImpl, 0x200);
assert_offset!(NxD3DImpl, driver, 0x180);

assert_offset!(NxD3DImpl_vtbl, fn_get_command_queue, 264);

impl_instance!(
    NxD3DImpl,
    "48 8B 0D ? ? ? ? 8B D3 4C 8B 01 41 FF 90 ? ? 00 00 48 81 C4 ? 01 00 00 5B C3"
);
