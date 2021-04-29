#include "asm.h"
.align 12
BEGIN_FUNC_FILE_NAME()
	li t1, 0x3000
	ld t1, (t1)
	li a7, HVC_VM_TEST_END
	ecall
END_FUNC_FILE_NAME()