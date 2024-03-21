#include <osdi/osdi.hpp>
#include <cstdint>

using namespace OSDI;

Partition::Partition(std::istream &stream)
{
    char buffer[16];
    
    stream.read(reinterpret_cast<char *>(&start), sizeof(uint32_t));
    stream.read(reinterpret_cast<char *>(&size), sizeof(uint32_t));
    stream.read(buffer, 8);
    type = std::string(buffer);
    stream.read(reinterpret_cast<char *>(&flags), 3);
    stream.read(buffer, 13);
    label = std::string(buffer);
}
