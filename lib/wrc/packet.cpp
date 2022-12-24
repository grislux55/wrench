#include <wrc/packet.h>

#include <cstdlib>

std::vector<uint8_t> WRCPacket::encode()
{
    uint8_t *data_ptr = (uint8_t *)(&this->data);
    uint8_t *encoded = NULL;
    size_t encoded_len = ::encode(encoded, data_ptr, sizeof(this->data));
    std::vector<uint8_t> bit_arr(encoded_len);
    memmove(bit_arr.data(), encoded, encoded_len);
    free(encoded);
    return bit_arr;
}

WRCPacket::WRCPacket(const uint8_t *const in, size_t in_len)
{
    uint8_t *decoded = NULL;
    size_t decoded_len = ::decode(decoded, in, in_len);
    memmove(&this->data, decoded, decoded_len);
    free(decoded);
}

WRCPacket::WRCPacket(const std::vector<uint8_t> &in)
{
    uint8_t *decoded = NULL;
    size_t decoded_len = ::decode(decoded, in.data(), in.size());
    memmove(&this->data, decoded, decoded_len);
    free(decoded);
}