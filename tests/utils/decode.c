#include <utils/decode.h>
#include <assert.h>

int main() {
    uint8_t source[] = {0x00, 0xcb, 0x7f, 0xaf, 0x57, 0xe1, 0x02};
    uint8_t target[] = {0xca, 0xfe, 0xba, 0xbe};
    uint8_t *decoded = NULL;
    size_t decoded_len = decode(&decoded, source, 7);
    assert(decoded_len == 4);
    for (int i = 0; i < decoded_len; i++) {
        assert(decoded[i] == target[i]);
    }
    return 0;
}
