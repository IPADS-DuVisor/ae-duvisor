#include "guest_entry.h"

int enter_guest(struct VcpuCtx *ctx) {
    int result;

    result = enter_guest_asm(ctx);
    return result;
}
