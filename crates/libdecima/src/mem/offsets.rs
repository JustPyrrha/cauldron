use crate::mem::{find_pattern, get_module, offset_from_instruction};
use std::cell::OnceCell;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Offsets {
    pub(crate) addresses: HashMap<String, *const u8>,
}

impl Offsets {
    pub fn instance() -> &'static mut Offsets {
        unsafe {
            static mut OFFSETS: OnceCell<Offsets> = OnceCell::new();
            OFFSETS.get_mut_or_init(|| Offsets {
                addresses: HashMap::new(),
            })
        }
    }

    pub fn map_address(&mut self, name: &str, address: *const u8) {
        self.addresses.entry(String::from(name)).or_insert(address);
    }

    pub fn map_pattern(&mut self, name: &str, pattern: &str) {
        let (start, end) = get_module().unwrap();
        self.map_address(
            name,
            find_pattern(start as *mut u8, end - start, pattern).unwrap(),
        );
    }

    pub fn resolve(&mut self, name: &str) -> Option<&*const u8> {
        self.addresses.get(name)
    }

    pub fn resolve_t<T: Sized>(&mut self, name: &str) -> Option<*mut T> {
        Some((get_module().unwrap().0 + *self.addresses.get(name)? as usize) as *mut T)
    }

    pub fn setup(&mut self) {
        if !self.addresses.is_empty() {
            return;
        }

        #[cfg(feature = "hfw")]
        {
            self.map_address(
                "nx::NxLogImpl::Instance",
                offset_from_instruction(
                    "48 8B 1D ? ? ? ? 48 8B 03 48 8B 78 48 48 8B 0D ? ? ? ?",
                    3,
                )
                .unwrap(),
            );
            self.map_pattern(
                "nx::NxLogImpl::fn_log",
                "4C 89 44 24 18 4C 89 4C 24 20 53 56 57 41 56",
            );

            self.map_pattern(
                "RTTIFactory::RegisterType",
                "40 55 53 56 48 8D 6C 24 ? 48 81 EC ? ? ? ? 0F B6 42 05 48 8B DA 48 8B",
            );
            self.map_pattern(
                "RTTIFactory::RegisterAllTypes",
                "40 55 48 8B EC 48 83 EC 70 80 3D ? ? ? ? ? 0F 85 ? ? ? ? 48 89 9C 24",
            );
        }
    }
}
