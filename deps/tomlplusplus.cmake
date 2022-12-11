include(FetchContent)
FetchContent_Declare(
        tomlplusplus
        URL https://gitlab.com/someproj/tomlplusplus/-/archive/v3.2.0/tomlplusplus-v3.2.0.zip
)
FetchContent_MakeAvailable(tomlplusplus)
include_directories(${tomlplusplus_SOURCE_DIR})
