#ifndef WRENCH_COM_WRENCH_H
#define WRENCH_COM_WRENCH_H

#include <string>
#include <vector>

struct SerialPortInfo {
    std::string portName;
    std::string description;
};

std::vector<SerialPortInfo> query_system_com_port();

#endif