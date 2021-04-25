#include "csr.h"
#include "asm_offset.h"
#include "save_restore.S"

.align 2

.global enter_guest_asm

enter_guest_asm:
    /* a0 point to vcpu_ctx */

    /* save host gp with a0=ctx, except t0-t6 and zero-x0 */
    SAVE_HOST_CTX a0

    /* SSTATUS? HUSTATUS got hstatus.SPV & sstatus.SPP */
    ld t1, (GUEST_HYP_HUSTATUS)(a0)
    csrrw t1, CSR_HUSTATUS, t1
    sd t1, (HOST_HYP_HUSTATUS)(a0)

    /* SCOUNTEREN? HUCOUNTEREN should be 64-bit with hcounteren(32-bit) + scounteren(32-bit) */
    ld t1, (GUEST_HYP_HUCOUNTEREN)(a0)
    csrrw t1, CSR_HUCOUNTEREN, t1
    sd t1, (HOST_HYP_HUCOUNTEREN)(a0)


    /* save a0-vcpu-ctx in CSR_USCRATCH & save USCRATCH */ 
    csrrw t3, CSR_USCRATCH, a0
    sd t3, (HOST_HYP_USCRATCH)(a0)

    /* set utvec to catch the trap & save UTVEC */
    la	t4, __vm_exit
    csrrw t4, CSR_UTVEC, t4
    sd t4, (HOST_HYP_UTVEC)(a0)

    /* set UEPC */
    ld t0, (GUEST_HYP_UEPC)(a0)
    csrrw t0, CSR_UEPC, t0
    sd t0, (GUEST_HYP_UEPC)(a0)

    /* hufence */
    .word 0xE2000073

    /* restore guest GP except A0 & X0 */
    RESTORE_GUEST_CTX a0
    
    /* restore guest A0 */
    ld x10, (GUEST_GP_X10)(a0)

    /* huret */
    uret

    .align 2
    __vm_exit:

    /* save guest-a0 in sscratch & get host-a0 */
    csrrw a0, CSR_USCRATCH, a0

    /* save guest gp except A0 & X0 */
    SAVE_GUEST_CTX a0

    /* save guest A0 with USCRATCH */
    csrr t1, CSR_USCRATCH
    sd t1, (GUEST_GP_X10)(a0)

    /* save guest UEPC */
    csrr t0, CSR_UEPC
    sd t0, (GUEST_HYP_UEPC)(a0)

    /* restore host utvec */
    ld t1, (HOST_HYP_UTVEC)(a0)
    csrw CSR_UTVEC, t1

    /* restore host uscratch */
    ld t2, (HOST_HYP_USCRATCH)(a0)
    csrw CSR_USCRATCH, t2

    /* restore host HUCOUNTEREN */
    ld t3, (HOST_HYP_HUCOUNTEREN)(a0)
    csrrw t3, CSR_HUCOUNTEREN, t3
    sd t3, (GUEST_HYP_HUCOUNTEREN)(a0)

    /* restore host HUSTATUS */
    ld t4, (HOST_HYP_HUSTATUS)(a0)
    csrrw t4, CSR_HUSTATUS, t4
    sd t4, (GUEST_HYP_HUSTATUS)(a0)

    /* restore host gp with a0=ctx, except t0-t6 and zero-x0 */
    RESTORE_HOST_CTX a0

    /* Save CSR_UCAUSE & UTVAL for trap hadnler */
    csrr t3, CSR_UCAUSE
    sd t3, (GUEST_HYP_UCAUSE)(a0)
    csrr t3, CSR_UTVAL
    sd t3, (GUEST_HYP_UTVAL)(a0)

    ret

.global set_hugatp

set_hugatp:
    csrw CSR_HUGATP, a0
    ret
    