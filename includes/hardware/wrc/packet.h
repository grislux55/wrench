#ifndef WRENCH_HARDWARE_WRC_PACKET_H
#define WRENCH_HARDWARE_WRC_PACKET_H

#include <hardware/wrc/joint.h>
#include <hardware/wrc/payload.h>
#ifdef _MSC_VER
#define ATTR_PACKED
#pragma pack(push, 1)
#endif

enum WRC_Packet_Direction {
    WRC_Packet_Direction_FromClient = 0,  // 此数据包由客户端发至服务端
    WRC_Packet_Direction_FromServer = 1  // 此数据包由服务端发至客户端
};

enum WRC_Packet_Type {
    WRC_Packet_Type_Unknown = 0,
    WRC_Packet_Type_Info_Generic,
    WRC_Packet_Type_Info_Serial,
    WRC_Packet_Type_Info_Timing,
    WRC_Packet_Type_Info_Energy,
    WRC_Packet_Type_Info_Network,
    WRC_Packet_Type_GetInfo,
    WRC_Packet_Type_SetJoint,
    WRC_Packet_Type_SetWrenchTime,
    WRC_Packet_Type_GetJointData,
    WRC_Packet_Type_ClearJointData,
    WRC_Packet_Type_GetStatusReport,
    WRC_Packet_Type_Beep,
    WRC_Packet_Type_JointData,
    WRC_Packet_Type_StatusReport,
    WRC_Packet_Type_InlineJointData,
};

typedef union {
    WRC_Payload_Info_Generic info_general;
    WRC_Payload_Info_Serial info_serial;
    WRC_Payload_Info_Timing info_timing;
    WRC_Payload_Info_Energy info_energy;
    WRC_Payload_Info_Network info_network;
    WRC_Payload_SetJoint set_joint;
    WRC_Payload_SetWrenchTime set_wrench_time;
    WRC_Payload_GetJointData get_joint_data;
    WRC_Payload_GetInfo get_info;
    // TODO: Unknown payload type. CHECK documents.
    // WRC_Payload_JointData joint_data;
    WRC_Payload_InlineJointData inline_joint_data;
    WRC_Payload_StatusReport status_report;
    uint8_t data[240];  // payload 最长 240 字节
} WRC_Payload;

typedef struct {
    uint16_t sequence_id;      // 数据包序列号
    uint32_t mac;              // MAC 地址，客户端上电后随机生成
    uint8_t direction : 1;     // 方向，
    uint8_t variable_len : 1;  // 当前 payload 为变长（为 1 个 payload
                               // 长度的整数倍）
    uint8_t type : 6;          // 数据包类型
    uint8_t payload_len;       // payload 总长度
    WRC_Payload payload;
} ATTR_PACKED WRC_Packet;

#ifdef _MSC_VER
#pragma pack(pop)
#endif
#endif  // WRENCH_HARDWARE_WRC_PACKET_H
