#pragma once
#include "EntityComponent.hpp"
#include "PhysicsConstraintListener.hpp"

namespace HZR2
{
    class Destructibility : public EntityComponent, public PhysicsConstraintListener
    {
    public:
        virtual const RTTI *GetRTTI() const override;
        virtual ~Destructibility() override;

        uint8_t _pad0[0x18];
        bool invulnerable;
        bool die_at_zero_health;
    };
}
