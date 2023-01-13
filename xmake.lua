set_project("wrench")
set_xmakever("2.7.3")

set_warnings("allextra")
set_languages("c17", "c++20")

add_rules("mode.release", "mode.debug")

add_requires("argparse 2.6.0")
add_requires("gtest 1.12.1")
add_requires("hiredis 1.0.2")
add_requires("toml++ 3.2.0")
add_requires("spdlog v1.11.0")
add_requires("fmt 9.1.0")
add_requires("redis-plus-plus 1.3.7")

includes("lib", "src", "tests")
