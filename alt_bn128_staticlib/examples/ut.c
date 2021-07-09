#include <ctype.h>
#include <stdint.h>

extern uint32_t alt_bn128_add(char* data, uint32_t data_len, char* output);


int hex2bin(char *s, char *buf)
{
    int i,n = 0;
    for(i = 0; s[i]; i += 2) {
        int c = tolower(s[i]);
        if(c >= 'a' && c <= 'f')
            buf[n] = c - 'a' + 10;
        else buf[n] = c - '0';
        if(s[i + 1] >= 'a' && s[i + 1] <= 'f')
            buf[n] = (buf[n] << 4) | (s[i + 1] - 'a' + 10);
        else buf[n] = (buf[n] << 4) | (s[i + 1] - '0');
        ++n;
    }
    return n;
}

int main() {
    char buf0[1024] = {};
    char buf1[64] = {};
    char *inputs = "18b18acfb4c2c30276db5411368e7185b311dd124691610c5d3b74034e093dc9063c909c4720840cb5134cb9f59fa749755796819658d32efc0d288198f3726607c2b7f58a84bd6145f00c9c2bc0bb1a187f20ff2c92963a88019e7c6a014eed06614e20c147e940f2d70da3f74c9a17df361706a4485c742bd6788478fa17d7";
    char *expect = "2243525c5efd4b9c3d3c45ac0ca3fe4dd85e830a4ce6b65fa1eeaee202839703301d1d33be6da8e509df21cc35964723180eed7532537db9ae5e7d48f195c915";
    hex2bin(inputs, buf0);
    if (alt_bn128_add(buf0, 256, buf1) != 0) {
        return 1;
    }
    // hex2bin(expect, buf0);
    // for (int i = 0; i < 64; i++) {
    //     if (buf0[i] != buf1[i]) {
    //         return 1;
    //     }
    // }
}
