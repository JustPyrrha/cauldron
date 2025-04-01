#pragma once
#include "PCore.hpp"
#include "RTTIRefObject.hpp"

namespace HZR2
{
    class AssetPath
    {
        PCore::String path;
        uint32_t unk8 = 0xFFFFFFFF;
    };
    class StreamingRefProxy
    {
    public:
        struct Locator
        {
            AssetPath asset_path;
            PCore::GGUUID uuid;
            RTTIRefObject *object;
            uint32_t unk_28;
            uint32_t unk_30;
        };
        Locator *locator;
        uintptr_t flags_and_streaming_manager; // lower 52 = streaming manager instance*
    };
    
    class StreamingRefBase
    {
    private:
        StreamingRefProxy *proxy;
    public:
        StreamingRefBase() = default;
        StreamingRefBase(const StreamingRefBase&) = delete;
        StreamingRefBase& operator=(const StreamingRefBase&) = delete;
    };

    
    template<typename T>
    class StreamingRef : public StreamingRefBase
    {
    public:
        StreamingRef() = default;
        StreamingRef(const StreamingRef&) = delete;
        StreamingRef& operator=(const StreamingRef&) = delete;
    };
}
