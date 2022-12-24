#ifndef WRENCH_SM7BIT_DECODE_H
#define WRENCH_SM7BIT_DECODE_H

#include <cstddef>
#include <cstdint>

/**
 * @brief           将输入的字节序列中的第一段由控制指令包围的数据解码
 * @param[out]      out         输出的数据字节序列
 * @param[in]       in          输入的需要解码数据
 * @param[in]       in_len      输入数据长度
 * @return          解码后的数据长度，不能按规则提取出数据则为0
 * @note            函数会根据数据长度自动分配*out的空间
 *                  不会尝试覆盖输出变量的内容，如果*out不为NULL则什么都不会干
 * @par 示例
 * @code
 *    uint8_t source[] = {0x00, 0xcb, 0x7f, 0xaf, 0x57, 0xe1, 0x02};
 *    uint8_t target[] = {0xca, 0xfe, 0xba, 0xbe};
 *    uint8_t *decoded = NULL;
 *    size_t decoded_len = decode(decoded, source, 7);
 *    assert(decoded_len == 4);
 *    for (int i = 0; i < decoded_len; i++) {
 *        assert(decoded[i] == target[i]);
 *    }
 * @endcode
 **/
extern size_t decode(uint8_t *&out, const uint8_t *const in, size_t in_len);

#endif  // WRENCH_SM7BIT_DECODE_H
