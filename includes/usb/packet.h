#ifndef WRENCH_USB_PACKET_H
#define WRENCH_USB_PACKET_H

#include <hardware/usb/packet.h>
#include <sm7bit/decode.h>
#include <sm7bit/encode.h>

class USBLocalPacket : public IEncodable
{
   private:
    USBLocal_Packet data;

   public:
    virtual std::vector<uint8_t> encode();

    USBLocalPacket()
    {
        uint8_t *data_ptr = reinterpret_cast<uint8_t *>(&this->data);
        std::fill(data_ptr, data_ptr + sizeof(USBLocal_Packet), 0);
    }
    USBLocalPacket(const uint8_t *const in, size_t in_len);
    USBLocalPacket(const std::vector<uint8_t> &in);

    uint8_t get_type()
    {
        return this->data.type;
    }
    void set_type(uint8_t type)
    {
        this->data.type = type;
    }

    uint8_t get_payload_len()
    {
        return this->data.payload_len;
    }
    void set_payload_len(uint8_t payload_len)
    {
        this->data.payload_len = payload_len;
    }

    USBLocal_Payload &get_payload()
    {
        return this->data.payload;
    }
    void set_payload(const USBLocal_Payload &payload)
    {
        const uint8_t *data_ptr = reinterpret_cast<const uint8_t *>(&payload);
        uint8_t *dest_ptr = reinterpret_cast<uint8_t *>(&this->data.payload);
        std::copy(data_ptr, data_ptr + sizeof(USBLocal_Payload), dest_ptr);
    }
};

#endif  // WRENCH_USB_PACKET_H