#ifndef WRENCH_HARDWARE_WRC_JOINT_H
#define WRENCH_HARDWARE_WRC_JOINT_H

enum WRC_JointData_Mode {
    WRC_JointData_Mode_Torque = 0,   // "torque"
    WRC_JointData_Mode_Angle,        // "angle"
    WRC_JointData_Mode_TorqueAngle,  // "torque-angle"
    WRC_JointData_Mode_AngleTorque,  // "angle-torque"
};

enum WRC_JointData_Method {
    WRC_JointData_Method_Click = 0,  // "clic"
    WRC_JointData_Method_Peak,       // "peak"
    WRC_JointData_Method_Track,      // "trck"
};

enum WRC_JointData_Unit {
    WRC_JointData_Unit_Nm = 0,  // "Nm"
    WRC_JointData_Unit_inlb,    // "in lb"
    WRC_JointData_Unit_ftlb,    // "ft lb"
};

enum WRC_Status {
    WRC_Status_None = 0,
    WRC_Status_SetJoint_Success,     // 设置 Joint 成功
    WRC_Status_SetJoint_Failed,      // 设置 Joint 失败
    WRC_Status_JointsDeleted,        // 所有 Joint 已删除
    WRC_Status_GetJoint_Success,     // 获取 Joint 成功
    WRC_Status_GetJoint_RangeError,  // 获取 Joint 失败 - 范围错误
};

#endif  // WRENCH_HARDWARE_WRC_JOINT_H
