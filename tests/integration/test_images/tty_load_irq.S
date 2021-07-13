#include "asm.h"

BEGIN_FUNC_FILE_NAME()
	//li t3, 0x30000
    li t4, 0
    //sd t4, (t3)

	/* tty mmio */
	li t3, 0x3f8

    la   t1, irq_handler
    csrw stvec, t1
    csrs sstatus, 0x2
    li   t1, 0x200
    csrs sie, t1

	lb a0, 0x0(t3)
//
    li a1, 0xdead
	li a7, ECALL_VM_TEST_END
    ecall

loop:
	j loop
END_FUNC_FILE_NAME()

.align 4
irq_handler:
    // Count for the total number of vtimer
    //ld t4, (t3)
    addi t4, t4, 1
    //sd t4, (t3)

    // Linux think that ecall timer sbi will clear sip
	lb a0, 0x0(t3)

    li a1, 0xcafe
    //li a7, ECALL_VM_TEST_END
    //ecall

    li t5, 10
    beq t4, t5, end

	sret

end:
    li a7, ECALL_VM_TEST_END
    ecall

