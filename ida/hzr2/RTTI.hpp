#pragma once

#include <cstdint>
#include "PCore.hpp"

namespace HZR2
{
    enum class RTTIKind : uint8_t
    {
        Atom,
        Pointer,
        Container,
        Enum,
        Compound,
        EnumFlags,
        POD,
        EnumBitSet
    };

    enum RTTIFlags : uint8_t
    {
        RTTIFactory_Registered = 0x2,
        FactoryManager_Registered = 0x4
    };

    #pragma pack(push, 1)
    struct RTTI
    {
        int32_t id;
        RTTIKind kind;
        RTTIFlags factory_flags;
    };
    #pragma pack(pop)

    struct RTTIStringSpan
    {
        const char *data = nullptr;
        uint32_t length = 0;
    };

    struct RTTIIter
    {
        RTTI *container_type = nullptr;
        void *container = nullptr;
        uint32_t user_data = 0;
    };

    struct RTTIAtom : RTTI
    {
        uint16_t size;
        uint8_t alignment;
        bool is_simple;
        const char *type_name;
        const RTTIAtom *base_type;

        bool (*fn_from_string)(const RTTIStringSpan& span, void *out);
        bool (*fn_to_string)(const void *in, PCore::String &out);
        void *fn_unk_30;
        void (*fn_copy)(void *in, void *out);
        bool (*fn_equals)(const void *a, const void *b);
        void *(*fn_constructor)(const RTTI *type, void *object);
        void *(*fn_destructor)(const RTTI *type, void *object);
        bool (*fn_serialize)(void *in, void *out, bool swap_endian);
        bool (*fn_deserialize)(void *in, void *out);
        int32_t (*fn_get_serialized_size)(void *in);
        void (*fn_range_check)(const void *in, const char *min, const char *max);

        const RTTI *representation_type;
    };

    class RTTIContainer : public RTTI
    {
    public:
        struct Data
        {
            const char *type_name;
        };

        struct PointerData : Data
        {
            uint32_t size;
            uint32_t alignment;
            
            void (*fn_constructor)(const RTTI *type, void *object);
            void (*fn_destructor)(const RTTI *type, void *object);
            void *(*fn_getter)(const RTTI *type, const void *object);
            bool (*fn_setter)(const RTTI *type, void **out, void *in);
            void (*fn_copy)(void **out, void **in);
        };

        struct ContainerData : Data
        {
            uint16_t size;
            uint8_t alignment;
            bool simple;
            bool associative;

            void (*fn_constructor)(const RTTI *type, void *object);
            void (*fn_destructor)(const RTTI *type, void *object);
            bool (*fn_resize)(const RTTI *type, void *object, int32_t new_size, bool unk);
            void *fn_unk_28;
            bool (*fn_remove)(const RTTI *type, void *object, int32_t index);
            int32_t (*fn_len)(const RTTI *type, const void *object);
            RTTIIter (*fn_iter_start)(const RTTI *type, const void *object);
            RTTIIter (*fn_iter_end)(const RTTI *type, const void *object);
            void (*fn_iter_next)(const RTTIIter &iter);
            void *(*fn_iter_deref)(const RTTIIter &iter);
            bool (*fn_is_iter_valid)(const RTTIIter &iter);
            RTTIIter (*fn_add_item)(const RTTI *type, void *object, void *value);
            RTTIIter (*fn_add_empty)(const RTTI *type, void *object);
            bool (*fn_clear)(const RTTI *type, void *object);
            bool (*fn_to_string)(const void *object, const RTTI *type, const PCore::String &out);
            bool (*fn_from_string)(const RTTIStringSpan &span, const RTTI *type, const void *object);
        };

        bool has_pointers;
        RTTI *item_type;
        Data *container_type;
        const char *full_type_name;
    };

    class RTTIEnum : public RTTI
    {
    public:
        struct Value
        {
            int32_t value;
            const char *name;
            const char *aliases[4];
        };

        uint8_t size;
        uint16_t values_len;
        uint8_t alignment;
        const char *type_name;
        Value *values;
        RTTI *pod_optimised_type;
    };

    class RTTICompound : public RTTI
    {
    public:
        class Base
        {
        public:
            RTTI *type;
            uint32_t offset;
        };

        class Attribute
        {
        public:
            enum Flags : uint16_t
            {
                ATTR_DONT_SERIALIZE_BINARY = 2,
                ATTR_VALID_FLAG_MASK = 3563
            };

            RTTI *type;
            uint16_t offset;
            Flags flags;
            const char *type_name;

            void (*fn_get)(const void *a, void *b);
            void (*fn_set)(void *a, const void *b);
            const char *range_min;
            const char *range_max;
        };

        class OrderedAttribute : public Attribute
        {
        public:
            const RTTI *parent;
            const char *group;
        };

        class MessageHandler
        {
        public:
            RTTI *message;
            void (*fn_handler)(void *object, void *message);
        };

        class MessageOrderEntry
        {
        public:
            bool before;
            RTTI *message;
            RTTI *compound;
        };

        uint8_t bases_len;
        uint8_t attributes_len;
        uint8_t message_handler_len;
        uint8_t message_order_entries_len;
        uint8_t _pad0[0x3];
        uint16_t version;
        uint32_t size;
        uint16_t alignment;
        uint16_t serialize_flags;

        void *(*fn_constructor)(const RTTI *type, void *object);
        void *(*fn_destructor)(const RTTI *type, void *object);
        void (*fn_from_string)();
        void (*fn_to_string)();
        uint8_t _pad1[0x8];

        const char *type_name;
        uint32_t cached_type_name_hash;
        uint8_t _pad2[0xC];

        Base *bases;
        Attribute *attributes;
        MessageHandler *message_handlers;
        MessageOrderEntry *message_order_entries;

        const RTTI *(*fn_get_symbol_group)();
        RTTI *pod_optimised_type;

        const OrderedAttribute *ordered_attributes;
        uint32_t ordered_attributes_len;

        MessageHandler message_read_binary;
        uint32_t message_read_binary_offset;

        uint32_t unk0;
    };

    class RTTIPod : public RTTI
    {
    public:
        uint32_t size;
    };

    class RTTIBitSet : public RTTI
    {
    public:
        RTTI *type;
        const char *type_name;
    };
}
