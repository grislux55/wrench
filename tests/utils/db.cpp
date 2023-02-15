#include <gtest/gtest.h>
#include <hiredis/hiredis.h>
#include <sw/redis++/redis++.h>

TEST(UtilsDb, BasicConnect)
{
    using namespace sw::redis;
    redisContext *c = redisConnect("120.76.201.111", 6379);
    ASSERT_NE(c, nullptr);
    ASSERT_EQ(c->err, 0);
    redisFree(c);

    try {
        auto redis = Redis("tcp://120.76.201.111:6379");
    } catch (const Error &e) {
        FAIL() << "Cannot connect target ip";
    }
}
