# 拧紧系统对接扳手接口文档

所有消息都存放在 Redis 的消息队列中，队列名称: `jeecg_redis_topic`

Redis 的版本: Redis-x64-5.0.14.1
使用的 database: 10

1.  拧紧系统发送请求连接扳手

|字段|父级|描述|
| --- | --- | --- |
| `msgId` ||消息唯一标识(S+扳手序列号+时间yyyymmddhhMMssSSS+四位随机数)|
| `msgType` ||消息类型0=连接|
| `handlerName` || `TOPIC_WRENCH_CONNECTION` =扳手数据上行字符标识|
| `currentTime` ||发送时间|
| `msgTxt` ||消息内容的业务数据|
|· `stationIp` | `msgTxt` |工位IP|
|· `wrenchName` | `msgTxt` |扳手名称|
|· `wrenchSerial` | `msgTxt` |扳手序列号|
|· `taskId` | `msgTxt` |任务ID|

示例：

```json
{
    "msgId": "SZN-BS1000H13T121-20230119152002583-3743",
    "msgType": "0",
    "handlerName": "TOPIC_WRENCH_CONNECTION",
    "currentTime": "2023-01-19 15:20:02",
    "msgTxt": {
        "stationIp": "192.168.0.466",
        "taskId": "8047203892302",
        "wrenchName": "智能扳手50-250Nm",
        "wrenchSerial": "ZN-BS1000H13T121"
    }
}

```

2.  连接成功，扳手回写给拧紧系统

|字段|父级|描述|
| --- | --- | --- |
| `msgId` ||消息唯一ID，连接发送时传入的 `msgId` |
| `msgType` ||消息类型0=连接|
| `handlerName` || `TOPIC_WRENCH_CONNECTION_ASK` =扳手连接成功回写|
| `msgTxt` ||消息内容的业务数据|
|· `wrenchName` | `msgTxt` |扳手名称|
|· `wrenchSerial` | `msgTxt` |扳手序列号|
|· `status` | `msgTxt` |状态 0 = 连接成功 1=连接失败|
|· `desc` | `msgTxt` |描述|
|· `currentTime` | `msgTxt` |当前时间|
|· `taskId` | `msgTxt` |任务ID, 发送连接时的任务ID|

示例：

```json
{
    "msgType": "0",
    "msgId": "RZN-BS1000H13T121-20230119151638914-9707",
    "handlerName": "TOPIC_WRENCH_CONNECTION_ASK",
    "msgTxt": {
        "wrenchName": "扳手名称",
        "wrenchSerial": "ZN-BS1000H13T121",
        "taskId": "8047203892302",
        "status": "0",
        "desc": "连接成功",
        "currentTime": "2022-12-28 02:10:35",
    }
}
```

3.  拧紧系统发送任务数据给扳手

|字段|父级|描述|
| --- | --- | --- |
| `msgId` ||消息唯一标识(S+扳手序列号+时间yyyymmddhhMMssSSS+四位随机数)|
| `msgType` ||消息类型0=扳手数据上行|
| `handlerName` || `TOPIC_WRENCH_TASK_UP_SEND` =扳手数据上行字符标识|
| `currentTime` ||当前时间|
| `msgTxt` ||消息内容的业务数据|
|· `stationIp` | `msgTxt` |工位IP|
|· `taskId` | `msgTxt` |任务ID|
|· `taskDetailId` | `msgTxt` |子任务ID|
|· `taskDesc` | `msgTxt` |任务的描述|
|· `wrenchSerial` | `msgTxt` |扳手序号|
|· `wrenchSerialDesc` | `msgTxt` |扳手描述|
|· `userId` | `msgTxt` |操作人员ID|
|· `userDesc` | `msgTxt` |操作人员名称|
|· `controlMode` | `msgTxt` |控制方式|
|· `workMode` | `msgTxt` |工作方式|
|· `boltNum` | `msgTxt` |螺栓数量|
|· `repeatCount` | `msgTxt` |重复次数|
|· `target` | `msgTxt` |目标|
|· `monitor` | `msgTxt` |监控|
|· `torque` | `msgTxt` |扭矩|
|· `torqueDeviationUp` | `msgTxt` |扭矩上偏差|
|· `torqueDeviationDown` | `msgTxt` |扭矩下偏差|
|· `torqueAngleStart` | `msgTxt` |扭矩角度启始值|
|· `angle` | `msgTxt` |角度|
|· `angleDeviationUp` | `msgTxt` |角度上偏差|
|· `angleDeviationDown` | `msgTxt` |角度下偏差|
|· `unit` | `msgTxt` |单位|

示例：

```json
{
    "msgType": "0",
    "msgId": "SZN-BS1000H13T121-20230119152209649-5640",
    "handlerName": "TOPIC_WRENCH_TASK_UP_SEND",
    "currentTime": "2023-01-19 15:22:09",
    "msgTxt": {
        "stationIp": "192.168.0.225",
        "taskId": "1615625798604156930",
        "taskDetailId": "1615612143439187969",
        "taskDesc": "六角头螺栓M10*16",
        "wrenchSerial": "ZN-BS1000H13T121",
        "controlMode": "0",
        "workMode": "0",
        "boltNum": 4,
        "repeatCount": 1,
        "target": "50",
        "monitor": "10",
        "torque": "50",
        "torqueDeviationUp": "1",
        "torqueDeviationDown": "1",
        "torqueAngleStart": "1",
        "angle": "10",
        "angleDeviationUp": "2",
        "angleDeviationDown": "2",
        "unit": "0"
    }
}
```

4.  拧紧系统收到扳手的任务数据回写

|字段|父级|描述|
| --- | --- | --- |
| `msgId` ||消息唯一ID，发送时传入的 `msgId` |
| `msgType` ||消息类型0=扳手数据上行|
| `handlerName` || `TOPIC_WRENCH_TASK_UP_ASK` =表示回写需要处理的类型|
| `msgTxt` ||业务信息的内容|
|· `wrenchSerial` | `msgTxt` |扳手序列号|
|· `currentTime` | `msgTxt` |当前时间|
|· `status` | `msgTxt` |状态，0= 接受成功，1=接受失败|
|· `desc` | `msgTxt` |接受成功/接受失败|

示例：

```json
{
    "msgType": "0",
    "msgId": "SZN-BS1000H13T121-20230118172156494-7874",
    "handlerName": "TOPIC_WRENCH_TASK_UP_ASK",
    "msgTxt": {
        "wrenchSerial": "ZN-BS1000H13T121",
        "status": "0",
        "desc": "连接成功",
        "currentTime": "2022-12-28 02:10:35",
    }
}
```

5.  拧紧系统采集扳手作业数据

|字段|父级|描述|
| --- | --- | --- |
| `msgId` ||消息唯一ID(R+扳手序列号+时间yyyymmddhhMMssSSS+四位随机数)|
| `msgType` ||4=表示采集扳手的作业数据|
| `handlerName` || `TOPIC_WRENCH_WORK_COLLECTION_RECEIVE` =表示扳手作业数据采集|
| `msgTxt` ||业务信息的内容
|· `msgId` | `msgTxt` |发送时task的 `msgId` 的消息
|· `taskId` | `msgTxt` |任务ID|
|· `taskDetailId` | `msgTxt` |子任务ID|
|· `wrenchSerial` | `msgTxt` |扳手序列号|
|· `torque` | `msgTxt` |扭矩|
|· `angle` | `msgTxt` |角度|
|· `status` | `msgTxt` |状态0=通过，1=不通过|
|· `consumeTime` | `msgTx` |耗时|
|· `desc` | `msgTxt` |描述|
|· `startDate` | `msgTxt` |开始时间|
|· `endDate` | `msgTxt` |结束时间|
|· `workTime` | `msgTxt` |作业时间|
|· `currentTime` | `msgTxt` |当前时间|

示例：

```json
{
    "msgType": "4",
    "msgId": "RZN-BS1000H13T121-20230119154534409-0908",
    "handlerName": "TOPIC_WRENCH_WORK_COLLECTION_RECEIVE",
    "msgTxt": {
        "consumeTime": "443",
        "wrenchSerial": "867c577e-5db7-4e9f-b364-9249ea0c23d1",
        "endDate": "",
        "msgId": "SZN-BS1000H13T121-20230118172156494-7874",
        "torque": "1246",
        "angle": "303",
        "workTime": "2023-01-19 15:45:35",
        "taskId": "1611286561058508802",
        "startDate": "2023-01-19 15:45:35",
        "taskDetailId": "1611286561125617665",
        "status": "0",
        "desc": "通过",
        "currentTime": "2023-01-19 15:45:35"
    }
}
```

6.  拧紧系统采集扳手基本信息/充电状态/异常/断开连接数据（待完善）

|字段	|父级	|描述|
| --- | --- | --- |
| `msgId` ||消息唯一ID(R+扳手序列号+时间yyyymmddhhMMss+四位随机数)|
| `msgType` ||0 = 基本信息, 1 = 充电状态 2 = 异常 3 = 断开连接|
| `handlerName` || `TOPIC_WRENCH_OTHER_COLLECTION_RECEIVE` =表示采集扳手基本信息/充电信息/异常信息|
| `msgTxt` ||业务信息的内容|
|· `wrenchSerial` | `msgTxt` |扳手序列号|
|· `title` | `msgTxt` |错误标题(异常需要传参)|
|· `code` | `msgTxt` |错误编码(异常需要传参)|
|· `startDate` | `msgTxt` |开始时间(异常需要传参)|
|· `endDate` | `msgTxt` |结束时间(异常需要传参)|
|· `level` | `msgTxt` |错误级别(异常需要传参)|
|· `consumeTime` | `msgTxt` |耗时(异常需要传参)|
|· `useTime` | `msgTxt` |使用时长(扳手基础信息传参)|
|· `storageNum` | `msgTxt` |存储数量(扳手基础信息传参)|
|· `status` | `msgTxt` |状态|
|· `desc` | `msgTxt` |描述|
|· `currentTime` | `msgTxt` |当前时间|

示例：

* 基础信息

```json
{
    "msgType": "0",
    "msgId": "RZN-BS1000H13T121-20230119155016580-7170",
    "handlerName": "TOPIC_WRENCH_OTHER_COLLECTION_RECEIVE",
    "msgTxt": {
        "storageNum": "120",
        "wrenchSerial": "a6ae5c9a-2a72-42c4-be6f-5c61354efcf7",
        "useTime": "1094",
        "desc": "扳手基础数据发送",
        "status": "2",
        "currentTime": "2023-01-19 15:50:16"
    }
}

```

* 充电

```json
{
    "msgType": "1",
    "msgId": "RZN-BS1000H13T121-20230119155118574-7676",
    "handlerName": "TOPIC_WRENCH_OTHER_COLLECTION_RECEIVE",
    "msgTxt": {
        "wrenchSerial": "293475fa-b01e-401a-9e4e-eda1f8ad3f82",
        "desc": "扳手正在充电发送",
        "status": "2",
        "currentTime": "2023-01-19 15:51:18"
    }
}

```

* 异常

```json
{
    "msgType": "2",
    "msgId": "RZN-BS1000H13T121-20230119155545011-9773",
    "handlerName": "TOPIC_WRENCH_OTHER_COLLECTION_RECEIVE",
    "msgTxt": {
        "consumeTime": "4140",
        "code": "TTK-ERROR-8361",
        "wrenchSerial": "ZN-BS1000H13T121",
        "level": "120",
        "endDate": "2022-12-29 12:54:21",
        "title": "网络问题，任务参数设置错误，数据丢失",
        "startDate": "2022-12-27 01:12:10",
        "desc": "网络异常，任务数据错误，导致执行异常",
        "currentTime": "2023-01-19 15:51:18"
    }
}

```

* 断开连接

```json
{
    "msgType": "3",
    "msgId": "RZN-BS1000H13T121-20230119155118574-7676",
    "handlerName": "TOPIC_WRENCH_OTHER_COLLECTION_RECEIVE",
    "msgTxt":{
        "wrenchSerial": "293475fa-b01e-401a-9e4e-eda1f8ad3f82",
        "desc": "断开连接",
        "status": "2",
        "currentTime": "2023-01-19 15:51:18"
    }
}

```

7.  拧紧系统采集扳手数据消息后发送给扳手（待完善）

|字段|父级|描述|
| --- | --- | --- |
| `msgId` ||消息唯一ID（接收MsgId消息ID）
| `msgType` ||0=基本信息接收回写给扳手, 1=充电信息接收回写给扳手, 2=异常信息接收回写给扳手, 3=断开连接接收回写给扳手, 4=采集扳手作业数据回写给扳手|
| `handlerName` || `TOPIC_WRENCH_COLLECTION_BACK` |
| `msgTxt` ||业务信息的内容|
|· `stationIp` | `msgTxt` |工位IP|
|· `status` | `msgTxt` |0= 接受成功，1=接受失败|
|· `desc` | `msgTxt` |描述|
|· `currentTime` | `msgTxt` |当前时间|

示例：

```json
{
    "msgType": "1/2/3/4",
    "msgId": "RZN-BS1000H13T121-20230119155118574-7676",
    "handlerName": "TOPIC_WRENCH_COLLECTION_BACK",
    "msgTxt":{
        "stationIp": "192.168.0.225",
        "status": "0",
        "desc": "接收成功",
        "currentTime": "2023-01-19 15:51:18"
    }
}

```
