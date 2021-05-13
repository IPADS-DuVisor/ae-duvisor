#include "asm.h"

// Output "Hello Ecall\n" via opensbi
BEGIN_FUNC_FILE_NAME()
	// target char
	li t0, 100

	li a7, SBI_EXT_0_1_CONSOLE_GETCHAR

	//li t0, 0x3000
	//jalr t0
continue:
	ecall
	bne a0, t0, continue

	// test end
	li a7, ECALL_VM_TEST_END
	ecall
END_FUNC_FILE_NAME()
