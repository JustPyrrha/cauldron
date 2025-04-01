#pragma once

namespace HZR2
{
    class PhysicsConstraintListener
    {
    public:
        virtual ~PhysicsConstraintListener();
        virtual void OnConstraintBroken() = 0;
    };
}
