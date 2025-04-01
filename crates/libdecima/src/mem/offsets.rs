use crate::mem::{PatternSearchError, find_pattern, get_module};
use std::cell::OnceCell;
use std::collections::HashMap;
use std::ops::{Add, Sub};
use std::ptr::read_unaligned;

#[derive(Debug)]
pub struct Offsets {
    pub(crate) addresses: HashMap<String, *const u8>,
}

impl Offsets {
    fn instance() -> &'static mut Offsets {
        unsafe {
            static mut OFFSETS: OnceCell<Offsets> = OnceCell::new();
            OFFSETS.get_mut_or_init(|| Offsets {
                addresses: HashMap::new(),
            })
        }
    }

    pub fn map_address(name: &str, address: *const u8) {
        Offsets::instance()
            .addresses
            .entry(String::from(name))
            .or_insert(address);
    }

    pub fn map_pattern(name: &str, pattern: &str) {
        let (start, end) = get_module().unwrap();
        Offsets::map_address(
            name,
            find_pattern(start as *mut u8, end - start, pattern).unwrap(),
        );
    }

    pub fn map_offset(name: &str, offset: Offset) {
        Offsets::map_address(name, offset.as_offset());
    }

    pub fn resolve_raw(name: &str) -> Option<&*const u8> {
        Offsets::instance().addresses.get(name)
    }

    pub fn resolve<T: Sized>(name: &str) -> Option<*mut T> {
        Some(
            get_module()
                .unwrap()
                .0
                .add(*Offsets::instance().addresses.get(name)? as usize) as *mut T,
        )
    }

    pub fn setup() {
        if !Offsets::instance().addresses.is_empty() {
            return;
        }

        #[cfg(feature = "hfw")]
        {
            // use crate::mem::offset_from_instruction;
            // Offsets::map_pattern(
            //     "nx::NxAppImpl::fn_create_swap_chain",
            //     "48 89 5C 24 10 48 89 6C 24 18 56 57 41 56 48 83 EC ? 48 8B F1 49 8B F8",
            // );
            // Offsets::map_pattern(
            //     "nx::NxD3DImpl::fn_initialize",
            //     "48 89 5C 24 18 48 89 6C 24 20 56 57 41 54 41 56 41 57 48 83 EC ? 45 0F B6 F0",
            // );

            // Offsets::map_pattern(
            //     "nx::NxDXGIImpl::fn_present",
            //     "40 55 56 57 41 54 41 56 48 8D AC 24 00 FD FF FF",
            // );
            // Offsets::map_offset(
            //     "nx::NxD3DImpl::Instance",
            //     Offset::from_signature(
            //         "48 8B 0D ? ? ? ? 8B D3 4C 8B 01 41 FF 90 ? ? 00 00 48 81 C4 ? 01 00 00 5B C3",
            //     )
            //     .unwrap()
            //     .as_relative(7),
            // );
            // Offsets::map_offset(
            //     "nx::NxDXGIImpl::Instance",
            //     Offset::from_signature("48 8D 0D ? ? ? ? 66 89 68 08 48 89 08 40 88 68 0A 48 89 68 0C 48 89 68 18 48 89 68 20")
            //         .unwrap()
            //         .as_relative(7),
            // );
            //
            // Offsets::map_pattern(
            //     "RTTIFactory::RegisterType",
            //     "40 55 53 56 48 8D 6C 24 ? 48 81 EC ? ? ? ? 0F B6 42 05 48 8B DA 48 8B",
            // );
            //
            // Offsets::map_pattern(
            //     "RTTIFactory::RegisterAllTypes",
            //     "40 55 48 8B EC 48 83 EC 70 80 3D ? ? ? ? ? 0F 85 ? ? ? ? 48 89 9C 24",
            // );
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Offset(usize);

impl Offset {
    pub fn new(address: usize) -> Self {
        Offset(address)
    }

    pub fn from_signature(pattern: &str) -> Result<Self, PatternSearchError> {
        let (module_start, module_end) = get_module()?;
        let search = find_pattern(module_start as *mut _, module_end - module_start, pattern)?;
        Ok(Self::new(search as _))
    }

    pub fn as_adjusted(&self, offset: usize) -> Offset {
        let result = Offset(self.0.add(offset));
        result
    }

    pub fn as_nadjusted(&self, offset: usize) -> Offset {
        Offset(self.0.sub(offset))
    }

    pub fn as_relative(&self, instruction_length: usize) -> Offset {
        let rel_adjust = unsafe {
            std::mem::transmute::<usize, *mut u32>(
                self.0.add(instruction_length.sub(size_of::<u32>())),
            )
        };
        let rel_adjust = unsafe { read_unaligned(rel_adjust) } as usize;
        let result = Offset(self.0.add(rel_adjust.add(instruction_length)));
        result
    }

    pub fn as_ptr<T>(&self) -> *mut T {
        self.0 as *mut T
    }

    pub fn as_offset(&self) -> *mut u8 {
        self.0.sub(get_module().unwrap().0) as *mut _
    }
}
