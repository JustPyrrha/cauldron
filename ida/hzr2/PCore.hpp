#pragma once

#include <cstdint>
#include <utility>
#include <windows.system.threading.h>

namespace HZR2
{
    namespace PCore
    {
        template<typename T>
        class Array final
        {
        public:
            uint32_t count = 0;
            uint32_t capacity = 0;
            T *data = nullptr;
        };

        template<typename K, typename V>
        class HashMap final
        {
        public:
            struct HashMapEntry
            {
                std::pair<const K, V> data;
                uint32_t hash = 0;
            };
            HashMapEntry *data;
            uint32_t count;
            uint32_t capacity;

            HashMap() = delete;
            HashMap(const HashMap &) = delete;
            ~HashMap() = delete;
        };

        struct GGUUID
        {
            uint8_t data[16];
        };

        template<typename T>
        class HashSet final
        {
        public:
            struct HashSetEntry
            {
                uint32_t hash;
                T data;
            };

            HashSetEntry *data;
            uint32_t count;
            uint32_t capacity;
        };

        class SharedMutex
        {
        public:
            SRWLOCK lock;
        };

        template<typename T>
        class SharedLockProtected : public SharedMutex
        {
        public:
            T data;
        };

        template<typename T>
        class Ref
        {
        public:
            T *ptr;
        };

        class String final
        {
        public:
            const char *data = nullptr;
        };
    }
}
