include(GoogleTest)

file(GLOB_RECURSE TEST_FILE_LIST *.cpp)
foreach (TEST_FILE_PATH ${TEST_FILE_LIST})
    string(REGEX MATCH "tests/.*" RELATIVE_PATH ${TEST_FILE_PATH})
    string(REGEX REPLACE "tests/" "" RELATIVE_PATH ${RELATIVE_PATH})
    string(REGEX REPLACE ".cpp" "" TARGET_NAME ${RELATIVE_PATH})
    string(REGEX REPLACE "/" "_" TARGET_NAME ${TARGET_NAME})

    add_executable(test_${TARGET_NAME} ${RELATIVE_PATH})
    target_link_libraries(test_${TARGET_NAME} ${PROJECT_NAME}_lib)
    target_link_libraries(test_${TARGET_NAME} GTest::gtest GTest::gtest_main GTest::gmock GTest::gmock_main)
    gtest_discover_tests(test_${TARGET_NAME})
endforeach ()

# Additional libraries linking
target_link_libraries(test_utils_db hiredis::hiredis)
target_link_libraries(test_conf_parse tomlplusplus::tomlplusplus)
