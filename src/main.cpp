#include "configuration.h"
#include "pch.h"

std::string_view get_version()
{
    constexpr int version_major = 0;
    constexpr int version_minor = 0;
    constexpr int version_patch = 0;
    static const std::string version = fmt::format(
        FMT_COMPILE("{}.{}.{}"), version_major, version_minor, version_patch);
    return version;
}

int main(int argc, char* argv[])
{
#ifndef NDEBUG
    spdlog::set_level(spdlog::level::debug);
#endif  // NDEBUG

    constexpr std::string_view program_name = "wrench_deamon";
    argparse::ArgumentParser program(std::string{program_name},
                                     std::string{get_version()});

    program.add_argument("--config", "-c")
        .required()
        .help("specify the configuration file path");

    try {
        program.parse_args(argc, argv);
    } catch (const std::runtime_error& err) {
        fmt::print(stderr, "{}\n{}", err.what(), program.help().str());
        std::exit(1);
    }

    spdlog::debug("current version is {}", get_version());

    auto config_path = program.get("-c");
    spdlog::debug("configuration path is {}", config_path);

    AppConfig config;
    try {
        auto parsed_config = toml::parse_file(config_path);
        config = load_config(parsed_config);
    } catch (const std::runtime_error& err) {
        fmt::print(stderr, "configuration invalid, reason: {}", err.what());
        std::exit(1);
    }
}
