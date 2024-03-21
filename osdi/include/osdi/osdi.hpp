#pragma once
#include <string>
#include <istream>

namespace OSDI
{
    enum PartitionFlags
    {
        HAS_OS = 0x000001,                // Contains an OS
        HAS_BOOTLOADER = 0x000002,        // Contains a bootloader
        HAS_POSIX_PERMISSIONS = 0x000004, // Has POSIX permissions
        READ_ONLY = 0x000008,             // Read only
        HIDDEN = 0x000010,                // Hidden
        SYSTEM = 0x000020,                // System partition
        ZORYA_SPECIAL = 0x000040,         // Zorya special value
        UNMANAGED = 0x000080,             // Unmanaged FS emulation
        RAW = 0x000100,                   // Raw data
        ACTIVE = 0x000200,                // Active
        OEFI_HINT = 0x000C00,             // OEFI hint mask
    };

    struct Partition
    {
        int start = 0;
        int size = 0;
        std::string type;
        int flags = 0;
        std::string label;

        Partition() = default;
        Partition(std::istream &stream);
    };
}
