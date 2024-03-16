#pragma once
#include <cstdint>

struct OSDIPartition
{
    uint32_t start_sector;
    uint32_t size;
    char type[8];
    uint8_t flags[3]; // uint24_t is the unholiest thing to ever exist why do they exist
    char name[13];
};
