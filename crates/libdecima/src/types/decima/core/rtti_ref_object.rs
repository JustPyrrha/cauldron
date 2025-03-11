use crate::gen_with_vtbl;
use crate::types::decima::core::rtti::RTTI;
use crate::types::decima::p_core::prelude::*;

gen_with_vtbl!(
    RTTIRefObject,

    fn GetRTTI() -> *const RTTI;
    fn Destroy();
    fn GetReffedObjects();
    fn GetReffedObjects1();

    pub uuid: GGUUID,
    pub refs: u32,
);
