#include "core.h"
#include "pch.h"

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
