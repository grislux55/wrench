#include <utils/decode.h>

#include <cstdlib>
#include <cstring>

size_t decode(uint8_t *&out, const uint8_t *in, size_t in_len)
{
    // 传入不需要的数据的时候直接返回防止覆盖
    if (out != nullptr) {
        return 0;
    }

    // 找到需要解码的部分
    size_t start, end;
    for (start = 0; start < in_len; start++) {
        if (in[start] == 0) {
            break;
        }
    }
    for (end = start; end < in_len; end++) {
        if (in[end] == 2) {
            break;
        }
    }

    // 没有找到控制单元
    if (start >= in_len || end >= in_len || end <= start) {
        return 0;
    }

    // 根据内部数据的长度计算出需要多长的空间来存储解码后的数据
    const size_t data_len = end - start - 1;
    const size_t target_len = data_len * 7 / 8;

    // 没有需要解码的数据
    if (target_len <= 0) {
        return 0;
    }

    out = static_cast<uint8_t *>(std::malloc(target_len));
    memset(out, 0, target_len);

    size_t wrote_bit = 0;
    for (size_t i = start + 1; i < end; i++) {
        // 对于0110 1101 0110 1101这样的数据
        // 其偏移量分别为76543210
        // 从高位开始向uint8_t中写入，拿掉最低位的1
        // 0110 110 0110 110
        // 使用wrote_bit计数下标可以方便8位对齐
        // 0110 1100 1101 10
        for (int j = 7; j > 0; j--) {
            if (in[i] & (1 << j)) {
                // wrote_bit % 8 代表写入到了哪一位
                // 但是数字里面的位偏移是反过来的，所以用7减去
                out[wrote_bit / 8] |= 1 << (7 - (wrote_bit % 8));
            }
            wrote_bit++;
            // 写满数据之后直接返回，丢弃剩下用于对齐的数据
            if (wrote_bit / 8 >= target_len) {
                break;
            }
        }
        if (wrote_bit / 8 >= target_len) {
            break;
        }
    }

    return target_len;
}
