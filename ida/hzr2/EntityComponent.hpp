#pragma once

#include "Entity.hpp"
#include "RTTIRefObject.hpp"
#include "WeakPtr.hpp"

namespace HZR2
{
    class Entity;
    
    template<typename T>
    class EntityMessageProcessing
    {
    public:
    };
    
    class EntityComponentResource : public RTTIRefObject
    {
    public:
    };

    class EntityComponent : public RTTIRefObject, public WeakPtrRTTITarget, public EntityMessageProcessing<EntityComponent>
    {
    public:
        PCore::Ref<EntityComponentResource> resource;
        bool is_initialized;
        void *representation;
        Entity *entity;

        virtual const RTTI* GetRTTI() const override;
        virtual ~EntityComponent();
        virtual const RTTI *GetRepresentationType() const;
        virtual void SetEntity(Entity *entity);
        virtual void SetResource(EntityComponentResource *resource);
        virtual void UnknownEntityComponent07();
        virtual void UnknownEntityComponent08();
        virtual void UnknownEntityComponent09();
        virtual void UnknownEntityComponent10();
        virtual void *CreateNetState();
    };

    class EntityComponentContainer
    {
    public:
        PCore::Array<EntityComponent *> components;
        PCore::Array<uint16_t> component_types;
    };
}
