include(FetchContent)
#原仓库 https://github.com/redis/hiredis
FetchContent_Declare(
        hiredis
        URL https://gitlab.com/someproj/hiredis/-/archive/v1.0.0/hiredis-v1.0.0.zip
)
FetchContent_MakeAvailable(hiredis)
include_directories(${hiredis_SOURCE_DIR})
