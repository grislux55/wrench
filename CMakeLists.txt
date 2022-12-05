cmake_minimum_required(VERSION 3.24)
project(wrench C)

set(CMAKE_C_STANDARD 23)

add_compile_options(-Wall)
add_compile_options(-Wextra)
add_compile_options(-Weffc++)
add_compile_options(-Wno-sign-compare)
add_compile_options(-Wno-unused-function)
add_compile_options(-Wno-unused-const-variable)
add_compile_options(-Wno-unused-template)
add_compile_options(-Wno-c++98-compat)
add_compile_options(-Wno-c++98-compat-pedantic)
add_compile_options(-fstandalone-debug)

include_directories(includes)

file(GLOB_RECURSE ALL_LIBS src/*.c)

add_library(${PROJECT_NAME}_lib SHARED ${ALL_LIBS})

add_executable(${PROJECT_NAME} main.c)

target_link_libraries(${PROJECT_NAME} ${PROJECT_NAME}_lib)
