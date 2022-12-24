#ifndef WRENCH_WRC_PACKET_H
#define WRENCH_WRC_PACKET_H

#include <hardware/wrc/packet.h>
#include <sm7bit/decode.h>
#include <sm7bit/encode.h>

#include <cstring>

class WRCPacket : public IEncodable
{
   private:
    WRC_Packet data;

   public:
    virtual std::vector<uint8_t> encode();

    WRCPacket()
    {
        memset(&data, 0, sizeof(WRC_Packet));
    }
    WRCPacket(const uint8_t *const in, size_t in_len);
    WRCPacket(const std::vector<uint8_t> &in);

    uint16_t get_sequence_id()
    {
        return this->data.sequence_id;
    }
    void set_sequence_id(uint16_t sequence_id)
    {
        this->data.sequence_id = sequence_id;
    }

    uint32_t get_mac()
    {
        return this->data.mac;
    }
    void set_mac(uint32_t mac)
    {
        this->data.mac = mac;
    }

    uint8_t get_direction()
    {
        return this->data.direction;
    }
    void set_direction(uint8_t direction)
    {
        this->data.direction = direction & 1;
    }

    uint8_t get_variable_len()
    {
        return this->data.variable_len;
    }
    void set_variable_len(uint8_t variable_len)
    {
        this->data.variable_len = variable_len & 1;
    }

    uint8_t get_type()
    {
        return this->data.type;
    }
    void set_type(uint8_t type)
    {
        this->data.type = type & 0x3F;
    }

    uint8_t get_payload_len()
    {
        return this->data.payload_len;
    }
    void set_payload_len(uint8_t payload_len)
    {
        this->data.payload_len = payload_len;
    }

    WRC_Payload &get_payload()
    {
        return this->data.payload;
    }
    void set_payload(const WRC_Payload &payload)
    {
        memcpy(&this->data.payload, &payload, sizeof(WRC_Payload));
    }
};

#endif  // WRENCH_WRC_PACKET_H