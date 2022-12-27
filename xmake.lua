set_project("wrench")
set_xmakever("2.7.3")

set_warnings("allextra", "error")
set_languages("c17", "c++20")

add_rules("mode.release", "mode.debug")

add_cxxflags("-Wno-sign-compare", {tools = {"gcc", "clang"}})
add_cxxflags("-Wno-unused-function", {tools = {"gcc", "clang"}})
add_cxxflags("-Wno-unused-const-variable", {tools = {"gcc", "clang"}})

add_requires("hiredis 1.0.2", "toml++ 3.2.0", "gtest 1.12.1", "argparse 2.6.0")

includes("lib", "src", "tests")
