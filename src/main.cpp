#include <com/wrench.h>

#include <chrono>
#include <mutex>

#include "configuration.h"
#include "pch.h"

using namespace std::chrono_literals;

constexpr semver::version version{0, 0, 0};
constexpr std::string_view program_name = "wrench_deamon";

void make_arguments(argparse::ArgumentParser& program, int argc,
                    const char* const argv[])
{
    program.add_argument("--config", "-c")
        .required()
        .help("specify the configuration file path");

    try {
        program.parse_args(argc, argv);
    } catch (const std::runtime_error& err) {
        fmt::print(stderr, "{}\n{}", err.what(), program.help().str());
        std::exit(1);
    }
}

AppConfig extract_arguments(argparse::ArgumentParser& program)
{
    auto config_path = program.get("-c");
    spdlog::debug("configuration path is {}", config_path);

    AppConfig config;
    try {
        auto parsed_config = toml::parse_file(config_path);
        config = load_config(std::move(parsed_config));
    } catch (const std::runtime_error& err) {
        fmt::print(stderr, "configuration invalid, reason: {}", err.what());
        std::exit(1);
    }

    return config;
}

void run_app(AppConfig& config)
{
    std::mutex com_status_mutex;
    std::vector<SerialPortInfo> com_status;

    concurrencpp::runtime runtime;

    concurrencpp::timer com_query_timer = runtime.timer_queue()->make_timer(
        1s, 1s, runtime.thread_executor(), [&] {
            auto status = query_system_com_port();
            const std::lock_guard<std::mutex> lock(com_status_mutex);
            com_status.swap(status);
        });
}

int main(int argc, char* argv[])
{
#ifndef NDEBUG
    spdlog::set_level(spdlog::level::debug);
#endif  // NDEBUG
    spdlog::debug("current version is {}", version.to_string());

    argparse::ArgumentParser program(std::string{program_name},
                                     version.to_string());

    make_arguments(program, argc, argv);

    auto config = extract_arguments(program);

    run_app(config);
}
