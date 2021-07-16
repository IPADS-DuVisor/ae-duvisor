#include "asm.h"

BEGIN_FUNC_FILE_NAME()
	csrr a0, 0x105 // stvec
	csrr a0, 0xf14 // mhartid
	
	// test end
	li a7, ECALL_VM_TEST_END
	ecall
END_FUNC_FILE_NAME()