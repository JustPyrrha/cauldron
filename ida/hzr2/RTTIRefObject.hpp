#pragma once
#include "RTTIObject.hpp"

namespace HZR2
{
    class RTTIRefObject : public RTTIObject
    {
    public:
        PCore::GGUUID uuid;
        uint32_t flags_and_ref_count;

        virtual const RTTI *GetRTTI() const override;
        virtual ~RTTIRefObject() override;
        virtual void GetReferencedObjects1();
        virtual void GetReferencedObjects2();
    };
}
