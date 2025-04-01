#pragma once
#include "CameraEntity.hpp"
#include "Entity.hpp"
#include "RTTIRefObject.hpp"
#include "WeakPtr.hpp"

namespace HZR2
{
    class CameraEntityRef;
    class AIFaction;
    
    class Player : public RTTIRefObject, public WeakPtrRTTITarget
    {
    public:
        uint8_t _pad0[0x18];
        Entity *entity;
        uint8_t _pad1[0x10];
        AIFaction *faction;
        uint8_t _pad2[0x88];
        PCore::Array<CameraEntityRef> camera_stack;
        PCore::SharedMutex mutex;
        uint8_t _pad3[0x20];

        virtual const RTTI *GetRTTI() const override;
        virtual ~Player() override;
        virtual void UnknownPlayer04();
        virtual void UnknownPlayer05();
        virtual void UnknownPlayer06();
        virtual void UnknownPlayer07();
        virtual void UnknownPlayer08();
        virtual void UnknownPlayer09();
        virtual void UnknownPlayer10();
        virtual void UnknownPlayer11();
        virtual void UnknownPlayer12();
        virtual void UnknownPlayer13();
        virtual void UnknownPlayer14();
        virtual void UnknownPlayer15();
        virtual void UnknownPlayer16();
        virtual void UnknownPlayer17();
    };
}
