use crate::types::decima::core::rtti::{RTTI, RTTIPod, RTTIPointerData};
use crate::types::decima::p_core::prelude::{Array, HashMap, HashSet, SharedLockProtected};
use crate::{gen_with_vtbl, impl_instance};
use std::ffi::c_void;

// 00000000 struct /*VFT*/ FactoryManager_vtbl // sizeof=0x20
// 00000000     void (__thiscall *Constructor)(FactoryManager *this);
// 00000008     void (__thiscall *Destructor)(FactoryManager *this);
// 00000010     void (__thiscall *Register)(FactoryManager *this, const RTTI *rtti);
// 00000018     void (__thiscall *Unregister)(FactoryManager *this, const RTTI *rtti);
// 00000020 };
//
// 00000000 struct __cppobj FactoryManager // sizeof=0x78
// 00000000 {
// 00000000     FactoryManager_vtbl *__vftable;
// 00000008     HashSet__pcRTTI mTypes;
// 00000018     HashMap__pRTTIPod__uint32_t mPodTypes;
// 00000028     SharedLockProtected__HashSet__pcRTTI mLockedTypes;
// 00000040     HashMap__RTTIPointerData__HashMap__pcRTTI__pcRTTI mPointerTypes;
// 00000050     HashMap__pcRTTI__pVoid mUnk50;
// 00000060     SharedLockProtected__Array__pVoid mUnk60;
// 00000078 };

gen_with_vtbl!(
    FactoryManager,
    FactoryManagerVtbl,

    fn constructor();
    fn destructor();
    fn register(rtti: *const RTTI);
    fn unregister(rtti: *const RTTI);

    pub types: HashSet<*const RTTI>,
    pub pod_types: HashMap<*mut RTTIPod, u32>,
    pub locked_types: SharedLockProtected<HashSet<*const RTTI>>,
    pub pointer_types: HashMap<RTTIPointerData, HashMap<*const RTTI, *const RTTI>>,
    pub unk_50: HashMap<*const RTTI, *mut c_void>,
    pub unk_60: SharedLockProtected<Array<*mut c_void>>,
);

impl_instance!(
    FactoryManager,
    "48 8B 0D ? ? ? ? 48 89 54 24 ? 8B 42 F8 89 44 24 28 8B 42 F4 48 8D 54 24 ? 89 44 24 2C E8 ? ? ? ? 48 85 C0 74 0D 48 8B C8 E8"
);
