#include "asm.h"

BEGIN_FUNC_FILE_NAME()
    /* T0: tty 0x3f8 */
    /* T1: offset */
    /* T2: 0x3f8 + offset */
    /* T3: Answer */
    /* T4: input saved */
    /* T5: irq count */
    /* T6: data */
    /* A0: external interrupt for sie */
    /* A1: judge value */
    /* A2: address of irq handler */
    /* A4: tmp */
    /* A5: data offset  */
    /* A6: step count */
    /* A7: ecall id */

    /* Set irq handler */
    la a2, irq_handler
    csrw stvec, a2

    /* Tty mmio address */
    li t0, 0x3f8

    /* Irq count start with 0 */
    li t5, 0

    /* Data offset start with 0 */
    li a5, 0

    /* Step ount start with 0 */
    li a6, 0

    /* Enable interrupt */
    csrs sstatus, 0x2

    /* Enable external interrupt */
    li a0, 0x200
    csrs sie, a0

/* Get irq at this point */
irq_miss:
    li a1, 0xdead
	li a7, ECALL_VM_TEST_END
    ecall
END_FUNC_FILE_NAME()

.align 4
irq_handler:
    addi t5, t5, 1
    li a4, 1
    beq t5, a4, first_irq
    li a4, 2
    beq t5, a4, second_irq
    li a4, 3
    beq t5, a4, third_irq
    li a4, 4
    beq t5, a4, fourth_irq
    j failure

first_irq:
    /* 1. Load 0xc4 at offset 2 */
    li t1, 2
    li t3, 0xc4
    add t2, t0, t1
    lb t6, 0x0(t2)
    addi a6, a6, 1
    bne t6, t3, failure

    /* 2. Load 0x61 at offset 5 */
    li t1, 5
    li t3, 0x61
    add t2, t0, t1
    lb t6, 0x0(t2)
    addi a6, a6, 1
    bne t6, t3, failure

    /* 3. Load input char at offset 0 and save it to t4 */
    li t1, 0
    add t2, t0, t1
    /* This should clear first_irq */
    lb t6, 0x0(t2)
    mv t4, t6
    addi a6, a6, 1

    /* 4. Load 0x60 at offset 5 */
    li t1, 5
    li t3, 0x60
    add t2, t0, t1
    lb t6, 0x0(t2)
    addi a6, a6, 1
    bne t6, t3, failure

    /* 5. Load 0xb0 at offset 6 */
    li t1, 6
    li t3, 0xb0
    add t2, t0, t1
    lb t6, 0x0(t2)
    addi a6, a6, 1
    bne t6, t3, failure

    /* 6. Load 0xc1 at offset 2 */
    li t1, 2
    li t3, 0xc1
    add t2, t0, t1
    lb t6, 0x0(t2)
    addi a6, a6, 1
    bne t6, t3, failure

    /* 7. Store 0x07 at offset 1 */
    li t1, 1
    li t6, 0x07
    add t2, t0, t1
    addi a6, a6, 1
    /* This should cause second_irq */
    sb t6, 0x0(t2)

    sret
    
    j failure

second_irq:
    /* 8. Load 0xc2 at offset 2 */
    li t1, 2
    li t3, 0xc2
    add t2, t0, t1
    lb t6, 0x0(t2)
    addi a6, a6, 1
    bne t6, t3, failure

    /* 9. Load 0x60 at offset 5 */
    li t1, 5
    li t3, 0x60
    add t2, t0, t1
    lb t6, 0x0(t2)
    addi a6, a6, 1
    bne t6, t3, failure

    /* 10. Load 0xb0 at offset 6 */
    li t1, 6
    li t3, 0xb0
    add t2, t0, t1
    lb t6, 0x0(t2)
    addi a6, a6, 1
    bne t6, t3, failure

    /* 11. Store 0x05 at offset 1 */
    li t1, 1
    li t6, 0x05
    add t2, t0, t1
    addi a6, a6, 1
    /* This should clear second_irq */
    sb t6, 0x0(t2)

    /* 12. Load 0xc1 at offset 2 */
    li t1, 2
    li t3, 0xc1
    add t2, t0, t1
    lb t6, 0x0(t2)
    addi a6, a6, 1
    bne t6, t3, failure

    /* 13. Store 0x07 at offset 1 */
    li t1, 1
    li t6, 0x07
    add t2, t0, t1
    addi a6, a6, 1
    /* This should cause third_irq */
    sb t6, 0x0(t2)

    sret

    j failure

third_irq:
    /* 14. Load 0xc2 at offset 2 */
    li t1, 2
    li t3, 0xc2
    add t2, t0, t1
    lb t6, 0x0(t2)
    addi a6, a6, 1
    bne t6, t3, failure

    /* 15. Load 0x60 at offset 5 */
    li t1, 5
    li t3, 0x60
    add t2, t0, t1
    lb t6, 0x0(t2)
    addi a6, a6, 1
    bne t6, t3, failure

    /* 16. Load 0xb0 at offset 6 */
    li t1, 6
    li t3, 0xb0
    add t2, t0, t1
    lb t6, 0x0(t2)
    addi a6, a6, 1
    bne t6, t3, failure

    /* 17. Store input char 'Z' at offset 0 */
    li t1, 1
    mv t6, t4
    add t2, t0, t1
    addi a6, a6, 1
    sb t6, 0x0(t2)

    /* 18. Store 0x05 at offset 1 */
    li t1, 1
    li t6, 0x05
    add t2, t0, t1
    addi a6, a6, 1
    /* This should clear third_irq */
    sb t6, 0x0(t2)

    /* 19. Load 0xc1 at offset 2 */
    li t1, 2
    li t3, 0xc1
    add t2, t0, t1
    lb t6, 0x0(t2)
    addi a6, a6, 1
    bne t6, t3, failure

    /* 20. Store 0x07 at offset 1 */
    li t1, 1
    li t6, 0x07
    add t2, t0, t1
    addi a6, a6, 1
    /* This should cause fourth_irq */
    sb t6, 0x0(t2)

    sret

    j failure

fourth_irq:
    /* 21. Load 0xc2 at offset 2 */
    li t1, 2
    li t3, 0xc2
    add t2, t0, t1
    lb t6, 0x0(t2)
    addi a6, a6, 1
    bne t6, t3, failure

    /* 22. Load 0x60 at offset 5 */
    li t1, 5
    li t3, 0x60
    add t2, t0, t1
    lb t6, 0x0(t2)
    addi a6, a6, 1
    bne t6, t3, failure

    /* 23. Load 0xb0 at offset 6 */
    li t1, 6
    li t3, 0xb0
    add t2, t0, t1
    lb t6, 0x0(t2)
    addi a6, a6, 1
    bne t6, t3, failure

    /* 24. Store 0x05 at offset 1 */
    li t1, 1
    li t6, 0x05
    add t2, t0, t1
    addi a6, a6, 1
    /* This should clear fourth_irq */
    sb t6, 0x0(t2)

    /* 25. Load 0xc1 at offset 2 */
    li t1, 2
    li t3, 0xc1
    add t2, t0, t1
    lb t6, 0x0(t2)
    addi a6, a6, 1
    bne t6, t3, failure

    j success

failure:
    li a1, 0xbeef
	li a7, ECALL_VM_TEST_END
    ecall

success:
    li a1, 0xcafe
	li a7, ECALL_VM_TEST_END
    ecall

