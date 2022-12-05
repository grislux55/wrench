#ifndef WRENCH_ENCODE_H
#define WRENCH_ENCODE_H

#include <stddef.h>

// Note: out MUST BE NULL
extern size_t encode(void *out, void *in, size_t in_len);

#endif //WRENCH_ENCODE_H
