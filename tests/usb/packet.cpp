#include <gtest/gtest.h>
#include <usb/packet.h>

TEST(USBLocalPacket, BasicInit)
{
    USBLocalPacket pack;
    auto encoded = pack.encode();
    ASSERT_EQ(encoded[0], 0x00);
    for (size_t i = 1; i < encoded.size() - 2; i++) {
        ASSERT_EQ(encoded[i], 0x01);
    }
    ASSERT_EQ(encoded[encoded.size() - 2], 0x07);
    ASSERT_EQ(encoded.back(), 0x02);
}

TEST(USBLocalPacket, InitFromU8)
{
    USBLocalPacket pack;
    auto encoded = pack.encode();
    USBLocalPacket pack2(encoded.data(), encoded.size());
    auto encoded2 = pack2.encode();
    for (size_t i = 0; i < encoded.size(); i++) {
        ASSERT_EQ(encoded[i], encoded2[i]);
    }
}

TEST(USBLocalPacket, InitFromVector)
{
    USBLocalPacket pack;
    auto encoded = pack.encode();
    USBLocalPacket pack2(encoded);
    auto encoded2 = pack2.encode();
    for (size_t i = 0; i < encoded.size(); i++) {
        ASSERT_EQ(encoded[i], encoded2[i]);
    }
}
