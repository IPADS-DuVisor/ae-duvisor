#include "asm.h"

// Output "Hello Ecall\n" via opensbi
BEGIN_FUNC_FILE_NAME()
	// target char 'd'
	li a6, 100

// output "putchar succeed" from scripts/local/laputa_test.exp
continue:
	li a7, SBI_EXT_0_1_CONSOLE_GETCHAR
	ecall
	mv t0, a0
	li a7, SBI_EXT_0_1_CONSOLE_PUTCHAR
	ecall
	bne t0, a6, continue

	li a0, 10 // \n
	ecall

	// test end
	li a7, ECALL_VM_TEST_END
	ecall
END_FUNC_FILE_NAME()
