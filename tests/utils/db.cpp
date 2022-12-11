#include <cstdio>
#include <hiredis.h>

int main()
{
    redisContext *c = redisConnect("120.76.201.111", 6379);
    if (c == NULL || c->err) {
        if (c) {
            printf("Error: %s\n", c->errstr);
            // handle error
        } else {
            printf("Can't allocate redis context\n");
        }
    }
    return 0;
}
