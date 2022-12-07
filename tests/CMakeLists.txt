add_executable(utils_decode_test sm7bit/decode.cpp)
target_link_libraries(utils_decode_test ${PROJECT_NAME}_lib)
add_test(NAME utils_decode_test COMMAND utils_decode_test)

add_executable(utils_encode_test sm7bit/encode.cpp)
target_link_libraries(utils_encode_test ${PROJECT_NAME}_lib)
add_test(NAME utils_encode_test COMMAND utils_encode_test)
