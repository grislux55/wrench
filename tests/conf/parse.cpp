#include <gtest/gtest.h>

#include <filesystem>
#include <iostream>
#include <toml.hpp>

constexpr std::string_view DATA =
    "[database]\n"
    "ip = \"120.76.201.111\"\n"
    "ports = 6379";

TEST(ConfParse, BasicRead)
{
    auto config = toml::parse(DATA);
    std::string_view ip = config["database"]["ip"].value_or("");
    int64_t ports = config["database"]["ports"].value_or(0);
    ASSERT_EQ(ip, "120.76.201.111");
    ASSERT_EQ(ports, 6379);
}
