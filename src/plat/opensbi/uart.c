#include <stdio.h>
#include <unistd.h>

int getchar_emulation() {
    int nr = 0;
    int fd = 0;
    char a = '0';
    char *buf = &a;
    int count = 1;
    printf("Hello world\n");
    //while(nr == 0) {
        nr = read(fd, buf, count);
    //}
    printf("nr = %d\n", nr);
    printf("char a = %c\n", a);

    return a;
}