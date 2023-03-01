#ifndef WRENCH_DEAMON_MESSGE_H
#define WRENCH_DEAMON_MESSGE_H

#include <string>

// 拧紧系统发送请求连接扳手
typedef struct {
    std::string stationIp;
    std::string taskId;
    std::string wrenchName;
    std::string wrenchSerial;
} ConnectResquestMsg;

typedef struct {
    std::string msgId;
    std::string msgType;
    std::string handlerName;
    std::string currentTime;
    ConnectResquestMsg msgTxt;
} ConnectResqust;

// 扳手回写给拧紧系统
typedef struct {
    std::string wrenchName;
    std::string wrenchSerial;
    std::string status;
    std::string desc;
    std::string currentTime;
    std::string taskId;
} ConnectResponseMsg;

typedef struct {
    std::string msgType;
    std::string msgId;
    std::string handlerName;
    ConnectResponseMsg msgTxt;
} ConnectResponse;

// 拧紧系统发送任务数据给扳手
typedef struct {
    std::string stationIp;
    std::string taskId;
    std::string taskDetailId;
    std::string taskDesc;
    std::string wrenchSerial;
    std::string wrenchSerialDesc;
    std::string userId;
    std::string userDesc;
    std::string controlMode;
    std::string workMode;
    std::string boltNum;
    std::string repeatCount;
    std::string target;
    std::string monitor;
    std::string torque;
    std::string torqueDeviationUp;
    std::string torqueDeviationDown;
    std::string torqueAngleStart;
    std::string angle;
    std::string angleDeviationUp;
    std::string angleDeviationDown;
    std::string unit;
} TaskRequestMsg;

typedef struct {
    std::string msgId;
    std::string msgType;
    std::string handlerName;
    std::string currentTime;
    TaskRequestMsg msgTxt;
} TaskRequest;

// 拧紧系统收到扳手的任务数据回写
typedef struct {
    std::string wrenchSerial;
    std::string currentTime;
    std::string status;
    std::string desc;
} TaskResponseMsg;

typedef struct {
    std::string msgId;
    std::string msgType;
    std::string handlerName;
    TaskResponseMsg msgTxt;
} TaskResponse;

// 拧紧系统采集扳手作业数据
typedef struct {
    std::string msgId;
    std::string taskId;
    std::string taskDetailId;
    std::string wrenchSerial;
    std::string torque;
    std::string angle;
    std::string status;
    std::string consumeTime;
    std::string desc;
    std::string startDate;
    std::string endDate;
    std::string workTime;
    std::string currentTime;
} TaskStatusMsg;

typedef struct {
    std::string msgId;
    std::string msgType;
    std::string handlerName;
    TaskStatusMsg msgTxt;
} TaskStatus;

// TODO 拧紧系统采集扳手基本信息/充电状态/异常/断开连接数据
// TODO 拧紧系统采集扳手数据消息后发送给扳手
#endif  // WRENCH_DEAMON_MESSGE_H
