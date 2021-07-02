#include "asm.h"

BEGIN_FUNC_FILE_NAME()
	li a0, 0x3f8
	li a2, 0x48

	sb a2, 0x0(a1)

	// test end
	li a7, ECALL_VM_TEST_END
	ecall
END_FUNC_FILE_NAME()