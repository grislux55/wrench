#include <gtest/gtest.h>
#include <toml++/toml.h>

#include <filesystem>
#include <iostream>

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

TEST(ConfParse, FileRead)
{
    auto config = toml::parse_file("conf/resources/conf.toml");
    std::string_view ip = config["database"]["ip"].value_or("");
    int64_t ports = config["database"]["ports"].value_or(0);
    ASSERT_EQ(ip, "120.76.201.111");
    ASSERT_EQ(ports, 6379);
}
