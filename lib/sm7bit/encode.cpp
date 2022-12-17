#include <sm7bit/encode.h>

#include <cstdlib>
#include <cstring>

size_t encode(uint8_t *&out, const uint8_t *in, size_t in_len)
{
    // 传入不需要的数据的时候直接返回防止覆盖
    if (out != nullptr) {
        return 0;
    }

    // 根据内部数据的长度计算出需要多长的空间来存储编码后的数据
    const size_t target_len = (8 * in_len + 7 - 1) / 7 + 2;

    // 没有需要编码的数据
    if (target_len <= 0) {
        return 0;
    }

    out = static_cast<uint8_t *>(std::malloc(target_len));
    memset(out, 0, target_len);

    out[0] = 0x00;

    size_t wrote_bit = 8;
    for (size_t i = 0; i < in_len; i++) {
        // 对于1100 1010 1111 1110
        // 每从高位开始写入，每过七个位补充一位flag位（数据flag为1）
        // 分组 1100 101_ 0111 11__
        // 补充flag 1100 1011 0111 11x1
        for (int j = 7; j >= 0; j--) {
            if (wrote_bit % 8 == 7) {
                // wrote_bit % 8 代表写入哪一位
                // 每个字节的第8位值置1
                out[wrote_bit / 8] |= 1 << (7 - (wrote_bit % 8));
                wrote_bit++;
            }
            if (in[i] & (1 << j)) {
                out[wrote_bit / 8] |= 1 << (7 - (wrote_bit % 8));
            }
            wrote_bit++;
        }
    }

    // 空位用1填充
    while (wrote_bit / 8 != target_len - 1) {
        out[wrote_bit / 8] |= 1 << (7 - (wrote_bit % 8));
        wrote_bit++;
    }

    out[target_len - 1] = 0x02;

    return target_len;
}
