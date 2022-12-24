#ifndef WRENCH_HARDWARE_USB_PACKET_H
#define WRENCH_HARDWARE_USB_PACKET_H

#include <cstdint>
#define ATTR_PACKED __attribute__((packed))

enum USBLocal_Packet_Type {
    USBLocal_Packet_Type_Unknown = 0,
    USBLocal_Packet_Type_RFStatus,   // RF 状态
    USBLocal_Packet_Type_RFControl,  // RF 控制
    USBLocal_Packet_Type_MACMode,  // MAC 模式（调试用，一般不需要修改）
};

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

typedef struct {
    uint8_t type;         // 类型
    uint8_t payload_len;  // payload 长度
    union {
        USBLocal_Payload_RFStatus rf_status;
        USBLocal_Payload_RFControl rf_control;
        USBLocal_Payload_MACMode mac_mode;
    } payload;
} ATTR_PACKED USBLocal_Packet;

#endif  // WRENCH_HARDWARE_USB_PACKET_H
