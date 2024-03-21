#include <osdi/osdi.hpp>
#include <cstdint>
#include <cstring>
#include <algorithm>

using namespace OSDI;

Partition::Partition(std::istream &stream)
{
    char buffer[16];
    
    stream.read(reinterpret_cast<char *>(&start), sizeof(uint32_t));
    stream.read(reinterpret_cast<char *>(&size), sizeof(uint32_t));

    memset(buffer, 0, 9);
    stream.read(buffer, 8);
    type = std::string(buffer);

    stream.read(reinterpret_cast<char *>(&flags), 3);
    
    memset(buffer, 0, 14);
    stream.read(buffer, 13);
    label = std::string(buffer);
}

void Partition::write(std::ostream &stream)
{
    stream.write(reinterpret_cast<const char *>(&start), sizeof(uint32_t));
    stream.write(reinterpret_cast<const char *>(&size), sizeof(uint32_t));

    stream.write(type.c_str(), std::min<int>(type.size(), 8));
    stream.seekp(8 - type.size(), std::ios_base::cur);

    stream.write(reinterpret_cast<const char *>(&flags), 3);

    stream.write(label.c_str(), std::min<int>(label.size(), 13));
    stream.seekp(13 - label.size(), std::ios_base::cur);
}
