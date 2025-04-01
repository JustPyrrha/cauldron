#pragma once
#include "Entity.hpp"

namespace HZR2
{
    class CameraEntity : public Entity { };

    class CameraEntityRef : public WeakPtr<CameraEntity>
    {
    public:
        ~CameraEntityRef() = delete;

        uint8_t _pad0[0x68];
    };
}
