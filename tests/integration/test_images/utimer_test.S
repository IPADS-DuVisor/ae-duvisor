#include "asm.h"

// ECALL_CALL_FOR_UTIMER 0xE0
// ECALL_HANDLER_START 0xE1
// ECALL_RIGHT_CAUSE 0xE2
// ECALL_WRONG_CAUSE 0xE3
// ECALL_STEP_LOG 0xE4
// ECALL_STOP_UTIMER 0xE5


BEGIN_FUNC_FILE_NAME()
    /* Set up irq_handler */
    //la   a1, irq_handler
	li a1, 0x80200060 // Use the absolute address
    csrw stvec, a1

    /* Set up sstatus.SIE */
    csrs sstatus, 0x2

    /* Enable VS_EXT VS_TIMER VS_SOFT via SSIE bit */
    li   t1, 0x200
    csrs sie, t1
	li   t1, 0x20
    csrs sie, t1
	li   t1, 0x2
    csrs sie, t1
	li a7, ECALL_STEP_LOG // log 1
    ecall

call_utimer:
	csrr a5, time

    li a7, ECALL_CALL_FOR_UTIMER // Call ULH to insert utimer
    ecall

loop_forever:
	li a2, 0
	csrr a5, time
	li a7, ECALL_STEP_LOG // log N
    ecall

wait:
	add a2, a2, 1
	li a4, 900000000
	blt a2, a4, wait
    j loop_forever

protect:
    j protect

END_FUNC_FILE_NAME()

/* 
 * Irq handler for vcpu 0
 */
.align 4
irq_handler:
	li a7, ECALL_HANDLER_START // Tell ULH that irq has been triggered
    ecall

	csrr a5, scause // 0x8000000000000005 timer 0x8000000000000009 ext
	csrr a4, sie
	csrr a3, sip
	li t1, 0x8000000000000005

	li a7, ECALL_STOP_UTIMER // Stop utimer
	ecall

    beq	a5, t1, rightirq // if claim 0, this means claim 0 pending irq
	li a7, ECALL_WRONG_CAUSE
	ecall
    sret

rightirq:
    li a7, ECALL_RIGHT_CAUSE
    ecall
    sret

end:
	j end