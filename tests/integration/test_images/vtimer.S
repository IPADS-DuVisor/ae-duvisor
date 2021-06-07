#include "asm.h"


BEGIN_FUNC_FILE_NAME()
    csrw stvec, irq_handler
    csrs sstatus, 0x2
	li a7, SBI_EXT_0_1_SET_TIMER
	li a0, 0 // raise irq immediately
	ecall

    li a0, 1
    li a7, ECALL_VM_TEST_END
	ecall
END_FUNC_FILE_NAME()


irq_handler:
    li a0, 0xcafe
    li a7, ECALL_VM_TEST_END
	ecall
