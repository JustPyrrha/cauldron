#pragma once
#include "RTTI.hpp"

namespace HZR2
{
    class RTTIObject
    {
    public:
        virtual const RTTI *GetRTTI() const;
        virtual ~RTTIObject();
    };
}
