#include "csr.h"
#include "../vcpu/asm_offset.h"
#include "save_restore.S"

.align 2

.global enter_guest

enter_guest:
    /* a0 point to vcpu_ctx */

    /* save host gp with a0=ctx, except t0-t6 and zero-x0 */
    SAVE_HOST_CTX a0

    /* save a0-vcpu-ctx in CSR_USCRATCH & save USCRATCH */ 
    csrrw t3, CSR_USCRATCH, a0
    sd t3, HOST_HYP_USCRATCH(a0)

    ld t0, HOST_HYP_HUSTATUS(a0)
    csrw CSR_HUSTATUS, t0

    ld t0, HOST_HYP_HUCOUNTEREN(a0)
    csrw CSR_HUCOUNTEREN, t0

    /* Restore UEPC & UCAUSE & UTVAL for trap handler */
    ld t0, HOST_HYP_UEPC(a0)
    csrw CSR_UEPC, t0

    ld t0, HOST_HYP_UCAUSE(a0)
    csrw CSR_UCAUSE, t0

    ld t0, HOST_HYP_UTVAL(a0)
    csrw CSR_UTVAL, t0

    /* restore guest GP except A0 & X0 */
    RESTORE_GUEST_CTX a0
    
    /* restore guest A0 */
    ld x10, GUEST_GP_X10(a0)

    /* huret */
    uret

    .align 2
exit_guest:
    /* save guest-a0 in sscratch & get host-a0 */
    csrrw a0, CSR_USCRATCH, a0

    /* save guest gp except A0 & X0 */
    SAVE_GUEST_CTX a0

    /* save guest A0 with USCRATCH */
    csrr t1, CSR_USCRATCH
    sd t1, GUEST_GP_X10(a0)

    csrr t0, CSR_HUSTATUS
    sd t0, HOST_HYP_HUSTATUS(a0)

    csrr t0, CSR_HUCOUNTEREN
    sd t0, HOST_HYP_HUCOUNTEREN(a0)

    /* Save UEPC & UCAUSE & UTVAL for trap handler */
    csrr t0, CSR_UEPC
    sd t0, HOST_HYP_UEPC(a0)

    csrr t0, CSR_UCAUSE
    sd t0, HOST_HYP_UCAUSE(a0)

    csrr t0, CSR_UTVAL
    sd t0, HOST_HYP_UTVAL(a0)

    /* restore host gp with a0=ctx, except t0-t6 and zero-x0 */
    RESTORE_HOST_CTX a0

    ret

.global set_hugatp

set_hugatp:
    csrw CSR_HUGATP, a0
    ret
    
.global set_utvec

set_utvec:
    la t4, exit_guest
    csrw CSR_UTVEC, t4
    ret
