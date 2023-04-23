#include "asm.h"

BEGIN_FUNC_FILE_NAME()
    /* Set up irq_handler */
    //la   a1, irq_handler
	li a1, 0x802000a0 // Use the absolute address
    csrw stvec, a1

    /* Set up sstatus.SIE */
    csrs sstatus, 0x2

	// Enable hwirq 35 = 0x23
    li t0, 0xc00008c // Priority addr 
	li t1, 1
	sw t1, 0x0(t0)
	li a7, ECALL_VM_TEST_END // 1
    ecall

	//li t0, 0xc002084 // Enable addr enable base 0x2000 percpu 0x80
	//li t1, 8
	//sw t1, 0x0(t0)
	li t0, 0xc002104 // Enable addr 
	li t1, 8
	sw t1, 0x0(t0)
	//li t0, 0xc002184 // Enable addr 
	//li t1, 8
	//sw t1, 0x0(t0)
	//li t0, 0xc002204 // Enable addr 
	//li t1, 8
	//sw t1, 0x0(t0)
	li a7, ECALL_VM_TEST_END // 2
    ecall

    /* Enable VS_EXT via SSIE bit */
    li   t1, 0x200
    csrs sie, t1
	li   t1, 0x20
    csrs sie, t1
	li   t1, 0x2
    csrs sie, t1
	li a7, ECALL_VM_TEST_END // 3
    ecall

call_virq:
	csrs sstatus, 0x2

    /* Enable VS_SOFT via SSIE bit */
    li   t1, 0x2
    csrs sie, t1

	csrr a5, time

    li a7, ECALL_CALL_FOR_VIRQ // Call ULH to insert virq
	li a3, 0xdeadbeef
    ecall

	li t4, 0
wait:
	add t4, t4, 1
	li t5, 30000
	blt t4, t5, wait
    j call_virq

protect:
    j protect

END_FUNC_FILE_NAME()

/* 
 * Irq handler for vcpu 0
 */
.align 4
irq_handler:
	li a7, ECALL_ENTER_HANDLER // Tell ULH that irq has been triggered
    ecall
    li t0, 0xc202004 // claim addr

	

    lw t2, 0x0(t0) // t2 device id
	li a7, ECALL_CLAIM_FINISH
	ecall

//	// wait for utimer
//	li t4, 0
//wait_utimer:
//	add t4, t4, 1
//	li t5, 300000
//	blt t4, t5, wait_utimer

	mv a3, t2

	csrr a5, time
	csrr a2, scause
	csrr a4, sip

	csrr a6, scause
	li t1, 0x8000000000000005
	beq	a6, t1, utimerirq

    li t1, 0
	li t3, 35
    beq	t2, t1, utimerirq // if claim 0, this means claim 0 pending irq
    //beq	t2, t1, badirq // if claim 0, this means claim 0 pending irq
	bne	t2, t3, wrongirq
	//mv a3, t2
	sw a3, 0x0(t0) // t2 device id

	li a7, ECALL_HANDLER_FINISH
    ecall
    sret

badirq:
    li a7, ECALL_VM_TEST_END
    ecall
    sret

wrongirq:
	li a7, ECALL_WRONG_IRQ
	mv a3, t2
    ecall
    sret

utimerirq:
	li a7, ECALL_STOP_UTIMER
	ecall
	sret

end:
	j end