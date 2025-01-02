#include <Pattern16.h>

#include "pat16.h"

namespace Pat16 {
    const void* scan(const void* start, size_t end, const char* signature) {
        return Pattern16::scan(
            start,
            end - reinterpret_cast<uintptr_t>(start),
            signature
        );
    }
}