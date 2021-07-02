#include <stdio.h>
#include <unistd.h>

int getchar_emulation() {
    int fd = 0;
    char a = '0';
    char *buf = &a;
    int count = 1;
    read(fd, buf, count);

    return a;
}