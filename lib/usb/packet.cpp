#include <usb/packet.h>

std::vector<uint8_t> USBLocalPacket::encode()
{
    uint8_t *data_ptr = reinterpret_cast<uint8_t *>(&this->data);
    uint8_t *encoded = nullptr;
    size_t encoded_len = ::encode(encoded, data_ptr, sizeof(this->data));
    std::vector<uint8_t> bit_arr(encoded_len);
    std::copy(encoded, encoded + encoded_len, bit_arr.begin());
    delete[] encoded;
    return bit_arr;
}

USBLocalPacket::USBLocalPacket(const uint8_t *const in, size_t in_len)
{
    uint8_t *data_ptr = reinterpret_cast<uint8_t *>(&this->data);
    uint8_t *decoded = nullptr;
    size_t decoded_len = ::decode(decoded, in, in_len);
    std::copy(decoded, decoded + decoded_len, data_ptr);
    delete[] decoded;
}

USBLocalPacket::USBLocalPacket(const std::vector<uint8_t> &in)
{
    uint8_t *data_ptr = reinterpret_cast<uint8_t *>(&this->data);
    uint8_t *decoded = nullptr;
    size_t decoded_len = ::decode(decoded, in.data(), in.size());
    std::copy(decoded, decoded + decoded_len, data_ptr);
    delete[] decoded;
}