#pragma once
#include "EntityComponent.hpp"
#include "WorldTransform.hpp"

namespace HZR2
{
    class Mover : public EntityComponent
    {
    public:
        virtual const RTTI *GetRTTI() const override;
        virtual ~Mover() override;
        virtual bool IsActive();
        virtual void SetActive(bool active);
        virtual void OverrideMovement(const WorldTransform &transform, float duration, bool);
        virtual void UpdateOverrideMovementTarget(const WorldTransform& transform);
        virtual bool IsMovementOverridden();
        virtual void StopOverrideMovement();
        virtual float GetOverrideMovementDuration();
        virtual float GetOverrideMovementTime();
        virtual float GetOverrideMovementSpeed();
        char _pad50[0x8];
    };
}
