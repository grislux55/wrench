add_executable(utils_decode_test utils/decode.c)
target_link_libraries(utils_decode_test ${PROJECT_NAME}_lib)
add_test(NAME utils_decode_test COMMAND utils_decode_test)
