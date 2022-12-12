#include <gtest/gtest.h>
#include <sm7bit/encode.h>

TEST(SM7BitEncode, BasicAssertions)
{
    uint8_t source[] = {0xca, 0xfe, 0xba, 0xbe};
    uint8_t target[] = {0x00, 0xcb, 0x7f, 0xaf, 0x57, 0xef, 0x02};
    uint8_t *encoded = nullptr;
    size_t encoded_len = encode(encoded, source, 4);
    ASSERT_EQ(encoded_len, 7);
    for (int i = 0; i < encoded_len; i++) {
        ASSERT_EQ(encoded[i], target[i]);
    }
}

TEST(SM7BitEncode, EmptyAssertions)
{
    uint8_t source[] = {};
    uint8_t target[] = {0x00, 0x02};
    uint8_t *encoded = nullptr;
    size_t encoded_len = encode(encoded, source, 0);
    ASSERT_EQ(encoded_len, 2);
    for (int i = 0; i < encoded_len; i++) {
        ASSERT_EQ(encoded[i], target[i]);
    }
}

TEST(SM7BitEncode, IllegalAssertions)
{
    uint8_t source[] = {0xca, 0xfe, 0xba, 0xbe};
    auto *encoded = (uint8_t *)std::malloc(1);
    size_t encoded_len = encode(encoded, source, 4);
    ASSERT_EQ(encoded_len, 0);
    free(encoded);
}
