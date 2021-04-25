#include <stdint.h>
struct VcpuCtx;

extern int enter_guest(struct VcpuCtx *ctx);
extern void set_hugatp(uint64_t hugatp);
extern void set_utvec();
