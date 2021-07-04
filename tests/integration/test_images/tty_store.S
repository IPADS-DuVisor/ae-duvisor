#include "asm.h"

BEGIN_FUNC_FILE_NAME()
	/* UART_DLM = 0xfe */
	li t0, 0x3f9
	li a2, 0xfe
	sb a2, 0x0(t0)

	/* UART_FCR = UART_FCR_CLEAR_XMIT | UART_FCR_CLEAR_RCVR */
	li t0, 0x3fa
	li a2, 0x6
	sb a2, 0x0(t0)

	/* UART_LCR = UART_LCR_DLAB */
	li t0, 0x3fb
	li a2, 0x80
	sb a2, 0x0(t0)

	/* UART_MCR = UART_MCR_OUT2 */
	li t0, 0x3fc
	li a2, 0x08
	sb a2, 0x0(t0)

	/* UART_SCR = 0x0 */
	li t0, 0x3ff
	li a2, 0x0
	sb a2, 0x0(t0)

	/* UART_IER = 0xff & 0xf */
	li t0, 0x3f9
	li a2, 0xff
	sb a2, 0x0(t0)

	// test end
	li a7, ECALL_VM_TEST_END
	ecall
END_FUNC_FILE_NAME()