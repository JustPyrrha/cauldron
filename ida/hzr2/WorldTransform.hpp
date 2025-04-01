#pragma once
#include "RotMatrix.hpp"
#include "WorldPosition.hpp"

namespace HZR2
{
    struct WorldTransform
    {
        WorldPosition position;
        RotMatrix rotation;
    };
}
