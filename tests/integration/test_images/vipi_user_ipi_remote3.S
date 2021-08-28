#include "asm.h"

BEGIN_FUNC_FILE_NAME()
    /* Address for sync */
    li t0, 0x3000

    li t1, 0
    li t2, 1

    beq	a0, t1, test_success
    beq	a0, t2, vcpu_1
    j test_failed

vcpu_0:
    j test_success

vcpu_1:
    /* Set sync data */
    sd t2, 0x0(t0)

    ld t3, 0x0(t0)
    li t4, 0x2
    bne t4, t3, vcpu_1
    

test_success:
    li a7, ECALL_VM_TEST_END
    ecall

test_failed:
    li a7, SBI_TEST_FAILED
    ecall

END_FUNC_FILE_NAME()
