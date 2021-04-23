struct VcpuCtx;

extern int enter_guest(struct VcpuCtx *ctx);
extern int enter_guest_asm(struct VcpuCtx *ctx);
