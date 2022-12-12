target("all_test", function()
    add_packages("hiredis")
    add_packages("gtest")
    add_packages("toml++")
    set_kind("binary")
    add_includedirs("../includes")
    add_deps("wrench_lib")
    add_files("./*.cpp","./*/*.cpp")
end)
