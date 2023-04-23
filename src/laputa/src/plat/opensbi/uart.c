#include <stdio.h>
#include <unistd.h>

int getchar_emulation() {
    char a;

    a = getchar();

    return a;
}

void writel(__uint32_t val, __uint64_t addr) {
    printf("0x%lx\n", addr);
    asm volatile("sw %0, 0(%1)" : : "r" (val), "r" (addr));
}