#include "asm.h"

BEGIN_FUNC_FILE_NAME()
	li t0, 0x3f8
	lb a0, 0x0(t0)

	// test end
	li a7, ECALL_VM_TEST_END
	ecall
END_FUNC_FILE_NAME()