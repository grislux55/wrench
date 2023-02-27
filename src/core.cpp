#include "core.h"

#include <com/wrench.h>

#include <chrono>
#include <set>

using namespace std::chrono_literals;

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

// An automic boolean to indicate if the program is exiting
std::atomic_bool exiting = false;

// a ctrl handler to toggle the exiting flag
BOOL WINAPI ctrl_handler(DWORD fdwCtrlType)
{
    switch (fdwCtrlType) {
        // Handle the CTRL-C signal.
        case CTRL_C_EVENT:
            exiting = true;
            spdlog::debug("ctrl-c detected, exiting");
            return FALSE;

        // CTRL-CLOSE: confirm that the user wants to exit.
        case CTRL_CLOSE_EVENT:
            exiting = true;
            spdlog::debug("ctrl-close detected, exiting");
            return FALSE;

        case CTRL_BREAK_EVENT:
            exiting = true;
            spdlog::debug("ctrl-break detected, exiting");
            return FALSE;

        case CTRL_LOGOFF_EVENT:
            exiting = true;
            spdlog::debug("ctrl-logoff detected, exiting");
            return FALSE;

        case CTRL_SHUTDOWN_EVENT:
            exiting = true;
            spdlog::debug("ctrl-shutdown detected, exiting");
            return FALSE;

        default:
            return FALSE;
    }
}

// a timer that read the system com port every second
// and update the com_status
concurrencpp::null_result query_com_timer(
    std::shared_ptr<concurrencpp::runtime> runtime,
    std::shared_ptr<concurrencpp::timer_queue> timer_queue,
    std::shared_ptr<concurrencpp::executor> executor)
{
    std::set<std::string> com_threads;

    while (true) {
        auto status = query_system_com_port();
        // debug output
        // for (const auto& port : status) {
        //     spdlog::debug("port {}", port.portName);
        // }
        for (const auto& port : status) {
            if (com_threads.find(port.portName) == com_threads.end()) {
                spdlog::debug("new com port detected: {}", port.portName);
                // TODO: Start a new thread to handle the com port
            }
        }

        com_threads.clear();
        for (const auto& port : status) {
            com_threads.insert(port.portName);
        }

        co_await timer_queue->make_delay_object(1000ms, executor);
    }
}

void run_app(AppConfig& config)
{
    if (SetConsoleCtrlHandler(ctrl_handler, TRUE)) {
        spdlog::debug("ctrl handler registered");
    } else {
        spdlog::error("ctrl handler registration failed");
    }

    std::shared_ptr<concurrencpp::runtime> runtime =
        std::make_shared<concurrencpp::runtime>();

    query_com_timer(runtime, runtime->timer_queue(),
                    runtime->thread_pool_executor());

    while (!exiting) {
    }
}