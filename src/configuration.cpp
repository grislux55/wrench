#include "configuration.h"

AppConfig load_config(toml::parse_result &&res)
{
    AppConfig config;

    // load redis ip
    auto redis_ip = res["database"]["ip"].value<std::string>();
    if (!redis_ip.has_value()) {
        throw std::runtime_error(
            "configuation file needs field: [database.ip], type [string]");
    }
    config.redis_ip = redis_ip.value();

    // load redis port
    auto redis_port = res["database"]["port"].value<int>();
    if (!redis_port.has_value()) {
        throw std::runtime_error(
            "configuation file needs field: [database.port], type [int]");
    }
    config.redis_port = redis_port.value();
    spdlog::debug("target redis is {}:{}", config.redis_ip, config.redis_port);

    // load redis queue
    auto redis_queue = res["database"]["queue"].value<std::string>();
    if (!redis_queue.has_value()) {
        throw std::runtime_error(
            "configuation file needs field: [database.queue], type [string]");
    }
    config.redis_queue = redis_queue.value();
    spdlog::debug("redis queue is {}", config.redis_queue);

    return config;
}