#include "asm.h"


BEGIN_FUNC_FILE_NAME()
    la   t1, irq_handler
    csrw stvec, t1
    csrs sstatus, 0x2
    li   t1, 0x20
    csrs sie, t1
	li a7, SBI_EXT_0_1_SET_TIMER
    // timer irq will be triggered immediately 
    li  a0, 0
	ecall

    // The loop will be finished before the second time irq occurred 
	li 	t1, 0
	li 	t2, 0x1000
continue:
	add  t1, t1, 1
	bne  t1, t2, continue

    li a7, ECALL_VM_TEST_END
	ecall
END_FUNC_FILE_NAME()

.align 4
irq_handler:
    // linux think that ecall timer sbi will clear sip
	li a7, SBI_EXT_0_1_SET_TIMER
    // timer irq will not be triggered in the vm
    li  a0, 0x100000000
    ecall
    
    li a0, 0xcafe
	sret
