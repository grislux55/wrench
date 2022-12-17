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
    argparse::ArgumentParser program(program_name, get_version());

    try {
        program.parse_args(argc, argv);
    } catch (const std::runtime_error& err) {
        std::cerr << err.what() << std::endl;
        std::cerr << program;
        std::exit(1);
    }
}
