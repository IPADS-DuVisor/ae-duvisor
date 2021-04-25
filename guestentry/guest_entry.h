#include <stdint.h>
struct VcpuCtx;

extern int enter_guest(struct VcpuCtx *ctx);
extern int enter_guest_asm(struct VcpuCtx *ctx);
extern void set_hugatp(uint64_t hugatp);
