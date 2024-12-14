#include <Pattern16.h>

#include "pat16.h"

namespace Pat16 {
    const void* scan(const void* start, size_t end) {
        return Pattern16::scan(
            start,
            end - reinterpret_cast<uintptr_t>(start),
            "FF FF FF FF [00000???]"
        );
    }
}