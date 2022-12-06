#include <assert.h>
#include <stdlib.h>
#include <utils/encode.h>

int main()
{
    uint8_t source[] = {0xca, 0xfe, 0xba, 0xbe};
    uint8_t target[] = {0x00, 0xcb, 0x7f, 0xaf, 0x57, 0xef, 0x02};

    uint8_t *encoded = NULL;
    size_t encoded_len = encode(&encoded, source, 4);
    assert(encoded_len == 7);
    for (int i = 0; i < encoded_len; i++) {
        assert(encoded[i] == target[i]);
    }

    uint8_t source1[] = {};
    uint8_t target1[] = {0x00, 0x02};

    free(encoded);
    encoded = NULL;
    encoded_len = encode(&encoded, source1, 0);
    assert(encoded_len == 2);
    for (int i = 0; i < encoded_len; i++) {
        assert(encoded[i] == target1[i]);
    }
    return 0;
}
