#include <gtest/gtest.h>
#include <hiredis/hiredis.h>

TEST(UtilsDb, BasicConnect)
{
    redisContext *c = redisConnect("120.76.201.111", 6379);
    ASSERT_NE(c, nullptr);
    ASSERT_EQ(c->err, 0);
    redisFree(c);
}
