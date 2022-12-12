set_project("wrench")
set_xmakever("2.7.3")

set_warnings("all", "error")
set_languages("c17", "c++20")

add_rules("mode.release", "mode.debug")

add_cxxflags("-Wall","-Wextra","-Wno-sign-compare","-Wno-unused-function","-Wno-unused-const-variable")

add_requires("hiredis 1.0.2")
add_requires("toml++ 3.2.0")
add_requires("gtest 1.12.1")

includes("src", "tests")
