#ifndef WRENCH_ENCODE_H
#define WRENCH_ENCODE_H

#include <stddef.h>
#include <stdint.h>

// Note: out MUST BE NULL
extern size_t encode(uint8_t **out, const uint8_t *in, size_t in_len);

#endif //WRENCH_ENCODE_H
