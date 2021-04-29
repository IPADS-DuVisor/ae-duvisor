#include "asm.h"

.align 12
BEGIN_FUNC_FILE_NAME()
	li a7, HVC_VM_TEST_END
	ecall
END_FUNC_FILE_NAME()
