#include "asm.h"

BEGIN_FUNC_FILE_NAME()
    li a7, SBI_TEST_GET_VCPU_ID
    ecall

    li t0, 0
    li t1, 1

    beq	a0, t0, vcpu_0
    beq	a0, t1, vcpu_1
    j test_failed

vcpu_0:
    li a7, SBI_TEST_SYNC_WAIT
    ecall

    li a7, SBI_TEST_HU_USER_IPI
    li a0, 0
    li a1, 1
    ecall
    j test_success

vcpu_1:
    li a7, SBI_TEST_SYNC_SET
    ecall

loop_0:
    j loop_0
    j test_success

test_success:
    li a7, SBI_TEST_SUCCESS
    ecall

test_failed:
    li a7, SBI_TEST_FAILED
    ecall
END_FUNC_FILE_NAME()
