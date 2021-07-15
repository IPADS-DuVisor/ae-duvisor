#include "asm.h"


BEGIN_FUNC_FILE_NAME()
    li t3, 0x30000
    li t4, 0
    sd t4, (t3)

    la   t1, irq_handler
    csrw stvec, t1
    csrs sstatus, 0x2
    li   t1, 0x20
    csrs sie, t1
	li a7, SBI_EXT_0_1_SET_TIMER
    // Timer irq will be triggered immediately 
    li  a0, 0
	ecall

// if there is vtimer lost, this test would be in this loop forever.
loop:
    li t6, 0xcafe
    j loop
END_FUNC_FILE_NAME()

.align 4
irq_handler:
    // Count for the total number of vtimer
    ld t4, (t3)
    addi t4, t4, 1
    sd t4, (t3)

    // Linux think that ecall timer sbi will clear sip
	li a7, SBI_EXT_0_1_SET_TIMER
    li  a0, 0x80000
    ecall

    li t5, 1000
    beq t4, t5, end

	sret

end:
    li a0, 0xcafe
    li a7, ECALL_VM_TEST_END
    ecall
    
