#ifndef WRENCH_HARDWARE_USB_PAYLOAD_H
#define WRENCH_HARDWARE_USB_PAYLOAD_H

#include <cstdint>
#ifdef __GNUC__
#define ATTR_PACKED __attribute__((packed))
#endif
#ifdef _MSC_VER
#define ATTR_PACKED
#pragma pack(push, 1)
#endif

typedef struct {
    int8_t rssi, snr, rscp;  // 信号强度，信噪比，解扩后的信号强度（LoRa）
} ATTR_PACKED USBLocal_Payload_RFStatus;

typedef struct {
    uint32_t freq_hz;  // 频率
    uint8_t rsvd;
    uint8_t txpower;  // 发射功率，单位 dBm，如模块带 PA 可能无效
    union {
        struct {
            uint8_t sf, bw, cr, ldro;  // LoRa 相关参数
        } lora;
    };
} ATTR_PACKED USBLocal_Payload_RFControl;

typedef struct {
    uint8_t mode;
} ATTR_PACKED USBLocal_Payload_MACMode;

#ifdef _MSC_VER
#pragma pack(pop)
#endif
#endif  // WRENCH_HARDWARE_USB_PAYLOAD_H