cmake_minimum_required(VERSION 3.24)
project(wrench)

set(CMAKE_C_STANDARD 23)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_MODULE_PATH "${CMAKE_SOURCE_DIR}/deps;${CMAKE_MODULE_PATH}")

add_compile_options(-Wall)
add_compile_options(-Wextra)
add_compile_options(-Wno-sign-compare)
add_compile_options(-Wno-unused-function)
add_compile_options(-Wno-unused-const-variable)

add_subdirectory(src)

include(hiredis)
include(tomlplusplus)

option(BUILD_TEST "Whether or not to build the tests" ON)
if (${BUILD_TEST})
    message(STATUS "Configuring tests")
    enable_testing()  # Enable testing only works in root scope
    add_subdirectory(tests)
endif ()
