#define _GNU_SOURCE
#include <assert.h>
#include <errno.h>
#include <fcntl.h>
#include <sched.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
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
    unsigned long value;
    int fd_ioctl = open_driver(IOCTL_DRIVER_NAME);
    
    if (ioctl(fd_ioctl, IOCTL_LAPUTA_GET_API_VERSION, &value) < 0) {
        perror("Error ioctl IOCTL_LAPUTA_GET_API_VERSION");
        return -1;
    }
    printf("Value is %lx\n", value);

    unsigned long sm_info[2];
    sm_info[0] = 0xdead000;
    sm_info[1] = 0x1000;
    if (ioctl(fd_ioctl, IOCTL_LAPUTA_REGISTER_SHARED_MEM, sm_info) < 0) {
        perror("Error ioctl IOCTL_LAPUTA_REGISTER_SHARED_MEM");
        return -1;
    }

    unsigned long deleg_info[2];
    deleg_info[0] = 1 << 7;
    deleg_info[1] = 1 << 2;
    if (ioctl(fd_ioctl, IOCTL_LAPUTA_REQUEST_DELEG, deleg_info) < 0) {
        perror("Error ioctl IOCTL_LAPUTA_REQUEST_DELEG");
        return -1;
    }

    unsigned long hustatus;
    unsigned long before, after = 0x123;
    int times = 100;
    printf("Repeat reading hustatus for %d times on 4 cores\n", times);
    for (int i = 0; i < times; i++) {
        cpu_set_t my_set;
        CPU_ZERO(&my_set);
        CPU_SET((size_t)(i % 4), &my_set);
        sched_setaffinity(0, sizeof(cpu_set_t), &my_set);
        asm volatile("csrr %0, 0x800" : "=r" (hustatus) :: "memory");

        asm volatile("csrrw %0, 0x40, %1" : "=r" (before) : "r" (after) : "memory");
        after = 0x456;
        asm volatile("csrrw %0, 0x40, %1" : "=r" (before) : "r" (after) : "memory");
        after = 0x789;
        asm volatile("csrrw %0, 0x40, %1" : "=r" (before) : "r" (after) : "memory");
        asm volatile("csrr %0, 0x40" : "=r" (after) :: "memory");
        assert(before == 0x456 && after == 0x789);
        
        sched_yield();
    }
    printf("hustatus = %lx\n", hustatus);
    printf("uscratch before = %lx, after = %lx\n", before, after);

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
