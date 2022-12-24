#ifndef WRENCH_HARDWARE_USB_PACKET_H
#define WRENCH_HARDWARE_USB_PACKET_H

#include <hardware/usb/payload.h>

#include <cstdint>

enum USBLocal_Packet_Type {
    USBLocal_Packet_Type_Unknown = 0,
    USBLocal_Packet_Type_RFStatus,   // RF 状态
    USBLocal_Packet_Type_RFControl,  // RF 控制
    USBLocal_Packet_Type_MACMode,  // MAC 模式（调试用，一般不需要修改）
};

typedef union {
    USBLocal_Payload_RFStatus rf_status;
    USBLocal_Payload_RFControl rf_control;
    USBLocal_Payload_MACMode mac_mode;
} USBLocal_Payload;

typedef struct {
    uint8_t type;         // 类型
    uint8_t payload_len;  // payload 长度
    USBLocal_Payload payload;
} ATTR_PACKED USBLocal_Packet;

#endif  // WRENCH_HARDWARE_USB_PACKET_H
