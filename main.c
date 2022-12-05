#include <stdio.h>
#include <utils/encode.h>
#include <utils/decode.h>

#define ARRAY_SIZE(__arr) (sizeof(__arr) / sizeof((__arr)[0]))

typedef unsigned char u8;

int main() {
    u8 source[] = {0xca, 0xfe, 0xba, 0xbe};
    u8 target[] = {0x00, 0xcb, 0x7f, 0xaf, 0x57, 0xe1, 0x02};

    u8 *encoded = NULL;
    int encoded_len = (int) encode((void *) encoded, (void *) source, ARRAY_SIZE(source));
    u8 *decoded = NULL;
    int decoded_len = (int) decode((void *) decoded, (void *) target, ARRAY_SIZE(target));

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
    printf("\nencoded len: %d\n", decoded_len);
    printf("data:");
    for (int i = 0; i < decoded_len; i++) {
        printf(" 0x%02X", decoded[i]);
    }

    return 0;
}
