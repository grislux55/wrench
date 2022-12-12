# 拧紧系统通信程序

## 编译指南

### 1. 环境配置

安装：[GCC](https://gcc-mcf.lhmouse.com/)，[XMake](https://xmake.io/)

### 2. 初始化编译环境

```shell
xmake f -p mingw -a x86_64 -y
```

### 3. 启动编译

```shell
xmake -y
```

编译产物在 `mingw\x86_64`中

### 4. 运行测试

```shell
xmake run all_test
```
