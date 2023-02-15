#ifndef WRENCH_DEAMON_CONFIGURATION_H
#define WRENCH_DEAMON_CONFIGURATION_H

#include <string>

#include "pch.h"

typedef struct {
    int redis_port;
    std::string redis_ip;
    std::string redis_queue;
} AppConfig;

extern AppConfig load_config(toml::parse_result &&res);

#endif  // WRENCH_DEAMON_CONFIGURATION_H