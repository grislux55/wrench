#ifndef WRENCH_HARDWARE_WRC_PAYLOAD_H
#define WRENCH_HARDWARE_WRC_PAYLOAD_H

#include <cstdint>
#define ATTR_PACKED __attribute__((packed))

typedef struct {
    uint8_t serial[16];  // 以二进制方式存储，节省空间和带宽
} ATTR_PACKED WRC_Payload_Info_Serial;

typedef struct {
    uint16_t joint_count;               // 当前的 Joint 数量
    uint16_t last_server_packet_seqid;  // 最后收到的服务端数据包的 sequence_id
} ATTR_PACKED WRC_Payload_Info_Generic;

typedef struct {
    uint32_t cpu_ticks;    // 处理器运行时间，单位为毫秒
    uint32_t wrench_time;  // 扳手中存储的时间，格式为 unix time
} ATTR_PACKED WRC_Payload_Info_Timing;

typedef struct {
    struct {
        uint16_t collisions, crc_errors;  // 碰撞，CRC 错误
        uint16_t tx_count, rx_wanted_count,
            rx_unwanted_count;  // 发送/接收有用/接收无用的包数量
    } packets;
    struct {
        int8_t rx_rssi, rx_snr, rx_rscp;  // 上次收到包的信号状态
    } rf;
} ATTR_PACKED WRC_Payload_Info_Network;

typedef struct {
    union {
        struct {
            uint8_t charging : 1;         // 正在充电
            uint8_t hibernated : 1;       // 休眠中
            uint8_t power_connected : 1;  // 电源已连接
            uint8_t f3 : 1;
            uint8_t f4 : 1;
            uint8_t f5 : 1;
            uint8_t f6 : 1;
            uint8_t f7 : 1;
        } bits;
        uint8_t value;
    } flag;
    uint16_t battery_voltage_mv;  // 电池电压，单位 mV
} ATTR_PACKED WRC_Payload_Info_Energy;

typedef struct {
    uint16_t joint_id;   // 自增的 ID
    uint16_t task_id;    // 所属的任务 ID
    uint32_t unix_time;  // 时间
    uint8_t valid : 1;   // 数据是否有效
    uint8_t ok : 1;      // 扳得是否正确
    uint8_t mode : 2;    // enum WRC_JointData_Mode
    uint8_t method : 2;  // enum WRC_JointData_Method
    uint8_t unit : 2;    // enum WRC_JointData_Unit
    int32_t torque;
    int16_t angle;
} ATTR_PACKED WRC_Payload_InlineJointData;

typedef struct {
    int32_t torque_setpoint, torque_angle_start, torque_upper_tol,
        torque_lower_tol;
    int16_t angle, angle_upper_tol, angle_lower_tol;
    int32_t fdt;
    int16_t fda;
    uint16_t task_repeat_times;  // 任务重复次数
    uint16_t task_id;            // 任务 ID
    uint8_t _rsvd : 2;
    uint8_t mode : 2;    // enum WRC_JointData_Mode
    uint8_t method : 2;  // enum WRC_JointData_Method
    uint8_t unit : 2;    // enum WRC_JointData_Unit
} ATTR_PACKED WRC_Payload_SetJoint;

typedef struct {
    uint32_t unix_time;
} ATTR_PACKED WRC_Payload_SetWrenchTime;

typedef struct {
    int16_t joint_id_start;  // 起始的 joint_id
    uint8_t joint_count;     // joint 数量
} ATTR_PACKED WRC_Payload_GetJointData;

typedef struct {
    uint16_t target_seqid;  // 服务端请求包中的 sequence_id
    uint16_t status;        // 返回值，一般 0 为没有错误
} ATTR_PACKED WRC_Payload_StatusReport;

typedef struct {
    union {
        struct {
            uint8_t serial : 1;
            uint8_t generic : 1;
            uint8_t energy : 1;
            uint8_t timing : 1;
            uint8_t network : 1;
            uint8_t f5 : 1;
            uint8_t f6 : 1;
            uint8_t f7 : 1;
        } bits;
        uint8_t value;
    } flag;
} ATTR_PACKED WRC_Payload_GetInfo;

#endif  // WRENCH_HARDWARE_WRC_PAYLOAD_H
