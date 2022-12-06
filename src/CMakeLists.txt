file(GLOB_RECURSE ALL_LIBS *.c)
add_library(${PROJECT_NAME}_lib ${ALL_LIBS})
target_include_directories(${PROJECT_NAME}_lib PUBLIC ../includes)
