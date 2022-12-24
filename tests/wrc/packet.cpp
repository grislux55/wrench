#include <gtest/gtest.h>
#include <wrc/packet.h>

TEST(WRCPacket, BasicInit)
{
    WRCPacket pack;
    auto encoded = pack.encode();
    ASSERT_EQ(encoded[0], 0x00);
    for (size_t i = 1; i < encoded.size() - 2; i++) {
        ASSERT_EQ(encoded[i], 0x01);
    }
    ASSERT_EQ(encoded[encoded.size() - 2], 0x1F);
    ASSERT_EQ(encoded.back(), 0x02);
}

TEST(WRCPacket, InitFromU8)
{
    WRCPacket pack;
    auto encoded = pack.encode();
    WRCPacket pack2(encoded.data(), encoded.size());
    auto encoded2 = pack2.encode();
    for (size_t i = 0; i < encoded.size(); i++) {
        ASSERT_EQ(encoded[i], encoded2[i]);
    }
}

TEST(WRCPacket, InitFromVector)
{
    WRCPacket pack;
    auto encoded = pack.encode();
    WRCPacket pack2(encoded);
    auto encoded2 = pack2.encode();
    for (size_t i = 0; i < encoded.size(); i++) {
        ASSERT_EQ(encoded[i], encoded2[i]);
    }
}
