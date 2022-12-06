#ifndef WRENCH_ENCODE_H
#define WRENCH_ENCODE_H

#include <stddef.h>
#include <stdint.h>

/**
 * @brief               将输入信息序列进行编码
 * @param out           输出的字节序列
 * @param in            输入的需要编码数据
 * @param in_len        输入数据长度
 * @return              编码后的数据长度
 * @note                函数会根据数据长度自动分配*out的空间
 *                      不会尝试覆盖输出变量的内容，如果*out不为NULL则什么都不会干
 * @par 示例
 * @code
 *     uint8_t source[] = {0xca, 0xfe, 0xba, 0xbe};
 *     uint8_t target[] = {0x00, 0xcb, 0x7f, 0xaf, 0x57, 0xef, 0x02};
 *
 *     uint8_t *encoded = NULL;
 *     size_t encoded_len = encode(&encoded, source, 4);
 *     assert(encoded_len == 7);
 *     for (int i = 0; i < encoded_len; i++) {
 *         assert(encoded[i] == target[i]);
 *     }
 * @endcode
 */

extern size_t encode(uint8_t **out, const uint8_t *in, size_t in_len);

#endif //WRENCH_ENCODE_H
