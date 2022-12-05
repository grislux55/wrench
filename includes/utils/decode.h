#ifndef WRENCH_DECODE_H
#define WRENCH_DECODE_H

#include <stddef.h>

// Note: out MUST BE NULL
extern size_t decode(void *out, void *in, size_t in_len);

#endif //WRENCH_DECODE_H
