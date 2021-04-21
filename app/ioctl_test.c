#define _GNU_SOURCE
#include <assert.h>
#include <errno.h>
#include <fcntl.h>
// #include <sched.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
// #include <sys/mman.h>
#include <sys/types.h>
#include <unistd.h>
#include <stdint.h>
#include <sys/ioctl.h>

#include "../linux-laputa/include/uapi/misc/laputa_dev.h"

#define IOCTL_DRIVER_NAME "/dev/laputa_dev"

int open_driver(const char* driver_name) {
    printf("* Open Driver\n");

    int fd_driver = open(driver_name, O_RDWR);
    if (fd_driver == -1) {
        printf("ERROR: could not open \"%s\".\n", driver_name);
        printf("    errno = %s\n", strerror(errno));
        exit(EXIT_FAILURE);
    }

    return fd_driver;
}

void close_driver(const char* driver_name, int fd_driver) {
    printf("* Close Driver\n");

    int result = close(fd_driver);
    if (result == -1) {
        printf("ERROR: could not close \"%s\".\n", driver_name);
        printf("    errno = %s\n", strerror(errno));
        exit(EXIT_FAILURE);
    }
}

int pass(void) {
    unsigned long deleg_info[2];
    int fd_ioctl = open_driver(IOCTL_DRIVER_NAME);
    deleg_info[0] = (1 << 20) | (1 << 21) | (1 << 23);
    deleg_info[1] = 1 << 2;
    if (ioctl(fd_ioctl, IOCTL_LAPUTA_REQUEST_DELEG, deleg_info) < 0) {
        perror("Error ioctl IOCTL_LAPUTA_REQUEST_DELEG");
        return -1;
    }

    printf("uret test for ULH\n");
    printf("uret test for ULH\n");
    printf("uret test for ULH\n");
    printf("uret test for ULH\n");
    printf("uret test for ULH %lx\n", pass);

#if 1
    asm volatile(
            "li t0, 0x200000180\n\t" //hustatus
            "csrw 0x800, t0\n\t"

            // "la t0, 1f\n\t" //utvec
            // "csrw 0x5, t0\n\t"

            "la t0, 1f\n\t" //uepc
            "csrw 0x41, t0 \n\t"

            "li t0, 0x0\n\t"
            "li t0, 0x8000000010000000\n\t"
            "csrw 0x880, t0\n\t" //hugatp
            // "csrw 0x480， t0\n\t" //huvsatp
            ".word 0xE2000073\n\t"
            "uret\n\t"
            "1:\n\t"
            ::: "t0"
        );
#endif

    close_driver(IOCTL_DRIVER_NAME, fd_ioctl);
    return 0;
}

int fail_ideleg(void) {
    int fd_ioctl = open_driver(IOCTL_DRIVER_NAME);
    
    unsigned long deleg_info[2];
    deleg_info[0] = 1 << 7;
    deleg_info[1] = 1 << 0;
    if (ioctl(fd_ioctl, IOCTL_LAPUTA_REQUEST_DELEG, deleg_info) < 0) {
        perror("Error ioctl IOCTL_LAPUTA_REQUEST_DELEG");
        close_driver(IOCTL_DRIVER_NAME, fd_ioctl);
        return 0;
    }

    close_driver(IOCTL_DRIVER_NAME, fd_ioctl);
    return -1;
}

int fail_edeleg(void) {
    int fd_ioctl = open_driver(IOCTL_DRIVER_NAME);
    
    unsigned long deleg_info[2];
    deleg_info[0] = 1 << 30;
    deleg_info[1] = 1 << 2;
    if (ioctl(fd_ioctl, IOCTL_LAPUTA_REQUEST_DELEG, deleg_info) < 0) {
        perror("Error ioctl IOCTL_LAPUTA_REQUEST_DELEG");
        close_driver(IOCTL_DRIVER_NAME, fd_ioctl);
        return 0;
    }

    close_driver(IOCTL_DRIVER_NAME, fd_ioctl);
    return -1;
}

int main(void) {
    int ret, nr_pass = 0, nr_fail = 0;
    ret = pass();
    if (ret) nr_fail++;
    else nr_pass++;
    
    ret = fail_ideleg();
    if (ret) nr_fail++;
    else nr_pass++;
    
    ret = fail_edeleg();
    if (ret) nr_fail++;
    else nr_pass++;

    printf("\n ------------ \n");
    if (nr_fail)
        printf("\nFAILED: [%d / %d] tests failed\n", nr_fail, nr_pass + nr_fail);
    else
        printf("\nPASSED: [%d / %d] tests passed\n", nr_pass, nr_pass + nr_fail);
    printf("\n ------------ \n");

    return EXIT_SUCCESS;
}