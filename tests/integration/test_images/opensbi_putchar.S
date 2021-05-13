#include "asm.h"

// print "Hello Ecall\n"
BEGIN_FUNC_FILE_NAME()
	li t0, 0
	li t1, 0

	li a0, 72 // H
	add t0, t0, a0
	li a7, SBI_EXT_0_1_CONSOLE_PUTCHAR
	ecall
	add t1, t1, a0

	li a0, 101 // e
	add t0, t0, a0
	li a7, SBI_EXT_0_1_CONSOLE_PUTCHAR
	ecall
	add t1, t1, a0

	li a0, 108 // l
	add t0, t0, a0
	li a7, SBI_EXT_0_1_CONSOLE_PUTCHAR
	ecall
	add t1, t1, a0

	li a0, 108 // l
	add t0, t0, a0
	li a7, SBI_EXT_0_1_CONSOLE_PUTCHAR
	ecall
	add t1, t1, a0

	li a0, 111 // o
	add t0, t0, a0
	li a7, SBI_EXT_0_1_CONSOLE_PUTCHAR
	ecall
	add t1, t1, a0

	li a0, 32 // blank
	add t0, t0, a0
	li a7, SBI_EXT_0_1_CONSOLE_PUTCHAR
	ecall
	add t1, t1, a0

	li a0, 69 // E
	add t0, t0, a0
	li a7, SBI_EXT_0_1_CONSOLE_PUTCHAR
	ecall
	add t1, t1, a0

	li a0, 99 // c
	add t0, t0, a0
	li a7, SBI_EXT_0_1_CONSOLE_PUTCHAR
	ecall
	add t1, t1, a0

	li a0, 97 // a
	add t0, t0, a0
	li a7, SBI_EXT_0_1_CONSOLE_PUTCHAR
	ecall
	add t1, t1, a0

	li a0, 108 // l
	add t0, t0, a0
	li a7, SBI_EXT_0_1_CONSOLE_PUTCHAR
	ecall
	add t1, t1, a0

	li a0, 108 // l
	add t0, t0, a0
	li a7, SBI_EXT_0_1_CONSOLE_PUTCHAR
	ecall
	add t1, t1, a0

	li a0, 10 // \n
	add t0, t0, a0
	li a7, SBI_EXT_0_1_CONSOLE_PUTCHAR
	ecall
	add t1, t1, a0

	// test end
	li a7, ECALL_VM_TEST_END
	ecall
END_FUNC_FILE_NAME()
