#include "asm.h"


BEGIN_FUNC_FILE_NAME()
    la   t1, irq_handler
    csrw stvec, t1
    csrs sstatus, 0x2
    li   t1, 0x20
    csrs sie, t1
	li a7, SBI_EXT_0_1_SET_TIMER
    // timer irq will be triggered immediately 
    csrr  a0, 0
	ecall

    li a0, 1
    li a7, ECALL_VM_TEST_END
	ecall
END_FUNC_FILE_NAME()

.align 4
irq_handler:
    li a0, 0xcafe
    li a7, ECALL_VM_TEST_END
	ecall
