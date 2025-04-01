#pragma once

namespace HZR2
{
    class PhysicsCollisionListener
    {
    public:
        virtual ~PhysicsCollisionListener();
        virtual bool OnPhysicsContactValidated();
        virtual void OnPhysicsContactAdded();
        virtual void OnPhysicsContactProcess();
        virtual void OnPhysicsContactRemoved();
    };
}
