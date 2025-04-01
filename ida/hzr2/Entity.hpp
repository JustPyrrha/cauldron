#pragma once
#include <atomic>
#include <mutex>

#include "Destructibility.hpp"
#include "EntityComponent.hpp"
#include "IStreamingManager.hpp"
#include "Mover.hpp"
#include "PhysicsCollisionListener.hpp"
#include "Player.hpp"
#include "RTTIRefObject.hpp"
#include "WeakPtr.hpp"
#include "WorldTransform.hpp"

namespace HZR2
{
    class AIFaction;
    class Model;
    class EntityResource : public RTTIRefObject { };
    class SpatialBins2DObject {};
    
    class Entity : public RTTIRefObject, public WeakPtrRTTITarget, public PhysicsCollisionListener, public SpatialBins2DObject
    {
    public:
        enum EFlags
        {
            None = 0,
            IsChanged = 0x1,
            IsVisible = 0x2,
            IsSleeping = 0x200,
        };

        uint8_t _pad0[0x30];
        StreamingRef<EntityResource> entity_resource;
        uint8_t _pad1[0x10];
        Entity *parent;
        Entity *first_child;
        Entity *last_sibling;
        std::atomic<EFlags> flags;
        EntityComponentContainer components;
        Mover *mover;
        Model *model;
        Destructibility *destructibility;
        WorldTransform transform;
        uint8_t _pad3[0x70];
        AIFaction *faction;
        uint8_t _pad4[0x108];
        mutable std::recursive_mutex lock;
        uint8_t _pad5[0x40];

        virtual const RTTI *GetRTTI() const override;
        virtual ~Entity() override;
        virtual WorldTransform GetSafePlacementPosition(const WorldTransform &pos);
        virtual void SetEntityResource(EntityResource *resource);
        virtual void UnknownEntity06();
        virtual void UnknownEntity07();
        virtual const Player *GetPlayer() const;
        virtual Player *GetPlayer();
        virtual void EnableCollision(bool, bool);
        virtual void UnknownEntity11();
        virtual void UnknownEntity12();
        virtual void UnknownEntity13();
        
        virtual bool OnPhysicsContactValidated() override;
        virtual void OnPhysicsContactAdded() override;
        virtual void OnPhysicsContactProcess() override;
        virtual void OnPhysicsContactRemoved() override;
    };
}
