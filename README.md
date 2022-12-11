# 拧紧系统通信程序

## 编译指南

### 1. 环境配置

安装：[GCC](https://gcc-mcf.lhmouse.com/)，[CMake](https://cmake.org/download/)，[Ninja](https://ninja-build.org/)，[Conan](https://conan.io/downloads.html)

### 2. 编译第三方包

1. 在项目根目录下新建一个build文件夹，Conan的编译信息将存放在其中
2. 依次执行如下命令

```shell
cd build
conan install .. --build missing -s compiler="gcc" --settings compiler.libcxx="libstdc++11"
```

### 3. 编译所有内容

```shell
cmake --force-config -DCMAKE_BUILD_TYPE=Debug -G Ninja ..
cmake --build . --clean-first --target all
```

程序运行文件将生成在`build`下

测试运行文件将生成在`build\tests`下

### 4. 运行测试

手动运行`build\tests`下的`*.exe`文件或执行如下命令：

```shell
ctest
```
