use crate::{assert_offset, assert_size, gen_with_vtbl};
use std::ffi::c_void;
use std::fmt::Debug;
use windows::Win32::Foundation::HMODULE;
use windows::Win32::Graphics::Dxgi::{IDXGIFactory1, IDXGISwapChain3};

gen_with_vtbl!(
    NxDXGIImpl,
    NxDXGIImplVtbl,

    fn fn_destructor();
    fn fn_initialize();
    fn fn_unk_10();
    fn fn_destroy_swap_chain();
    fn fn_resize_swap_chain();
    fn fn_is_swap_chain_full_screen();
    fn fn_set_swap_chain_full_screen();
    fn fn_unk_38();
    fn fn_set_color_space();
    fn fn_unk_48();
    fn fn_present(in_data: *mut c_void) -> bool;
    fn fn_unk_58();
    fn fn_unk_60();
    fn fn_unk_68();
    fn fn_unk_70();
    fn fn_unk_78();
    fn fn_unk_80();
    fn fn_get_dxgi_factory_1();
    fn fn_unk_90();
    fn fn_unk_98();
    fn fn_unk_a0();
    fn fn_unk_a8();
    fn fn_unk_b0();
    fn fn_unk_b8();

    pub initialized: bool,
    pub unk8: *mut c_void,
    pub dxgi_module: HMODULE,
    pub dxgi_factory_a: *mut IDXGIFactory1,
    pub dxgi_factory_b: *mut IDXGIFactory1,
    pub swap_chain: *mut IDXGISwapChain3,
    pub num_buffers: u32,
    pub pad_38: [u8;0xB4],
);

assert_size!(NxDXGIImpl, 0xf0);
assert_offset!(NxDXGIImpl, dxgi_factory_a, 0x20);
assert_offset!(NxDXGIImpl, dxgi_factory_b, 0x28);
assert_offset!(NxDXGIImpl, swap_chain, 0x30);
assert_offset!(NxDXGIImpl, num_buffers, 0x38);
