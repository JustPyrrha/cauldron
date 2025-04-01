#pragma once

namespace HZR2
{
    struct Vec3
    {
        float x;
        float y;
        float z;
    };

#pragma pack(push, 4)
    struct Vec3Pack
    {
        float x;
        float y;
        float z;
    };
#pragma pack(pop)
}
