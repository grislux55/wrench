#include <stdio.h>
#include <utils/encode.h>
#include <utils/decode.h>

#define ARRAY_SIZE(__arr) (sizeof(__arr) / sizeof((__arr)[0]))

int main() {
    uint8_t source[] = {0xca, 0xfe, 0xba, 0xbe};
    uint8_t target[] = {0x00, 0xcb, 0x7f, 0xaf, 0x57, 0xe1, 0x02};

    uint8_t *encoded = NULL;
    int encoded_len = (int) encode(&encoded, source, ARRAY_SIZE(source));
    uint8_t *decoded = NULL;
    int decoded_len = (int) decode(&decoded, target, ARRAY_SIZE(target));

    printf("original len: %llu\n", ARRAY_SIZE(source));
    printf("data:");
    for (int i = 0; i < ARRAY_SIZE(source); i++) {
        printf(" 0x%02X", source[i]);
    }
    printf("\ntargeting len: %llu\n", ARRAY_SIZE(target));
    printf("data:");
    for (int i = 0; i < ARRAY_SIZE(target); i++) {
        printf(" 0x%02X", target[i]);
    }

    printf("\n\nencoded len: %d\n", encoded_len);
    printf("data:");
    for (int i = 0; i < encoded_len; i++) {
        printf(" 0x%02X", encoded[i]);
    }
    printf("\ndecoded len: %d\n", decoded_len);
    printf("data:");
    for (int i = 0; i < decoded_len; i++) {
        printf(" 0x%02X", decoded[i]);
    }

    return 0;
}
