CROSS_COMPILE=riscv64-linux-gnu-
CC=$(CROSS_COMPILE)gcc
CFLAGS=-Wall -g -Wconversion -Wextra -static

all: clean ioctl_test

ioctl_test: ioctl_test.o
	$(CC) -o $@ $^ $(CFLAGS)

clean:
	rm -rf *.o ioctl_test
