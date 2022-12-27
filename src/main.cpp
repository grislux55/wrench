#include <spdlog/spdlog.h>
#include <toml++/toml.h>

#include <argparse/argparse.hpp>

constexpr char const* program_name = "wrench_deamon";
constexpr int version_major = 0;
constexpr int version_minor = 0;
constexpr int version_patch = 0;

std::string get_version()
{
    static const std::string version = std::to_string(version_major) + "." +
                                       std::to_string(version_minor) + "." +
                                       std::to_string(version_patch);
    return version;
}

int main(int argc, char* argv[])
{
#ifndef NDEBUG
    spdlog::set_level(spdlog::level::debug);
#endif  // NDEBUG

    static const auto version = get_version();
    argparse::ArgumentParser program(program_name, version);

    program.add_argument("--config", "-c")
        .required()
        .help("specify the configuration file path");

    try {
        program.parse_args(argc, argv);
    } catch (const std::runtime_error& err) {
        std::cerr << err.what() << std::endl;
        std::cerr << program;
        std::exit(1);
    }

    spdlog::debug("current version is {}", version);

    auto config_path = program.get("-c");
    auto config = toml::parse_file(config_path);
    spdlog::debug("configuration path is {}", config_path);
}
