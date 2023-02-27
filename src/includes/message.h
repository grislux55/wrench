#ifndef WRENCH_DEAMON_MESSGE_H
#define WRENCH_DEAMON_MESSGE_H

#include <string>

using namespace std;

// 拧紧系统发送请求连接扳手
typedef struct {
    string stationIp;
    string taskId;
    string wrenchName;
    string wrenchSerial;
} Connect_Resquest_Msg;

typedef struct {
    string msgId;
    string msgType;
    string handlerName;
    string currentTime;
    Connect_Resquest_Msg msgTxt;
} Connect_Resqust;

// 扳手回写给拧紧系统
typedef struct {
    string wrenchName;
    string wrenchSerial;
    string status;
    string desc;
    string currentTime;
    string taskId;
} Connect_Response_Msg;

typedef struct {
    string msgType;
    string msgId;
    string handlerName;
    Connect_Response_Msg msgTxt;
} Connect_Response;

// 拧紧系统发送任务数据给扳手
typedef struct {
    string stationIp;
    string taskId;
    string taskDetailId;
    string taskDesc;
    string wrenchSerial;
    string wrenchSerialDesc;
    string userId;
    string userDesc;
    string controlMode;
    string workMode;
    string boltNum;
    string repeatCount;
    string target;
    string monitor;
    string torque;
    string torqueDeviationUp;
    string torqueDeviationDown;
    string torqueAngleStart;
    string angle;
    string angleDeviationUp;
    string angleDeviationDown;
    string unit;
} Task_Request_Msg;

typedef struct {
    string msgId;
    string msgType;
    string handlerName;
    string currentTime;
    Task_Request_Msg msgTxt;
} Task_Request;

// 拧紧系统收到扳手的任务数据回写
typedef struct {
    string wrenchSerial;
    string currentTime;
    string status;
    string desc;
} Task_Response_Msg;

typedef struct {
    string msgId;
    string msgType;
    string handlerName;
    Task_Response_Msg msgTxt;
} Task_Response;

// 拧紧系统采集扳手作业数据
typedef struct {
    string msgId;
    string taskId;
    string taskDetailId;
    string wrenchSerial;
    string torque;
    string angle;
    string status;
    string consumeTime;
    string desc;
    string startDate;
    string endDate;
    string workTime;
    string currentTime;
} Task_Status_Msg;

typedef struct {
    string msgId;
    string msgType;
    string handlerName;
    Task_Status_Msg msgTxt;
} Task_Status;

// TODO 拧紧系统采集扳手基本信息/充电状态/异常/断开连接数据
// TODO 拧紧系统采集扳手数据消息后发送给扳手
#endif  // WRENCH_DEAMON_MESSGE_H
