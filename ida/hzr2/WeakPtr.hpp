#pragma once
#include "RTTI.hpp"

namespace HZR2
{
    class WeakPtrTarget;

    class WeakPtrBase
    {
    protected:
        WeakPtrTarget *ptr = nullptr;
        WeakPtrBase *prev = nullptr;
        WeakPtrBase *next = nullptr;
    public:
        WeakPtrBase() = delete;
        ~WeakPtrBase() = delete;
    };

    class WeakPtrTarget
    {
    public:
        WeakPtrBase *head = nullptr;
        virtual ~WeakPtrTarget();
    };

    class WeakPtrRTTITarget : public WeakPtrTarget
    {
    public:
        virtual ~WeakPtrRTTITarget() override;
        virtual const RTTI *GetRTTI() const;
    };

    template<typename T>
    class WeakPtr : protected WeakPtrBase
    {
    public:
        WeakPtr() = delete;
        ~WeakPtr() = delete;
    };
}
