#include <gtest/gtest.h>
#include <sm7bit/decode.h>

TEST(SM7BitDecode, BasicAssertions)
{
    uint8_t source[] = {0x00, 0xcb, 0x7f, 0xaf, 0x57, 0xe1, 0x02};
    uint8_t target[] = {0xca, 0xfe, 0xba, 0xbe};
    uint8_t *decoded = nullptr;
    size_t decoded_len = decode(decoded, source, 7);
    ASSERT_EQ(decoded_len, 4);
    for (size_t i = 0; i < decoded_len; i++) {
        ASSERT_EQ(decoded[i], target[i]);
    }
}

TEST(SM7BitDecode, EmptyAssertions)
{
    uint8_t *source = nullptr;
    uint8_t *decoded = nullptr;
    size_t decoded_len = decode(decoded, source, 0);
    ASSERT_EQ(decoded_len, 0);
    ASSERT_EQ(decoded, nullptr);
}

TEST(SM7BitDecode, IllegalAssertions)
{
    uint8_t source[] = {0x00, 0xcb, 0x7f, 0xaf, 0x57, 0xe1, 0x02};
    uint8_t *decoded = new uint8_t[1];
    size_t decoded_len = decode(decoded, source, 7);
    ASSERT_EQ(decoded_len, 0);
    free(decoded);
}
