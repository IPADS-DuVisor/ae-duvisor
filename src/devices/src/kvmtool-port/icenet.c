/* Can't add new CONFIG parameters in an external module, so define them here */
#define CONFIG_ICENET_MTU 1500
#define CONFIG_ICENET_RING_SIZE 64
#define CONFIG_ICENET_CHECKSUM
#define CONFIG_ICENET_TX_THRESHOLD 16

#define ICENET_NAME "icenet"
#define ICENET_SEND_REQ 0
#define ICENET_RECV_REQ 8
#define ICENET_SEND_COMP 16
#define ICENET_RECV_COMP 18
#define ICENET_COUNTS 20
#define ICENET_MACADDR 24
#define ICENET_INTMASK 32
#define ICENET_TXCSUM_REQ 40
#define ICENET_RXCSUM_RES 48
#define ICENET_CSUM_ENABLE 49

#define ICENET_INTMASK_TX 1
#define ICENET_INTMASK_RX 2
#define ICENET_INTMASK_BOTH 3

#define ETH_HEADER_BYTES 14
#define ALIGN_BYTES 8
#define ALIGN_MASK 0x7
#define ALIGN_SHIFT 3
#define MAX_FRAME_SIZE (CONFIG_ICENET_MTU + ETH_HEADER_BYTES + NET_IP_ALIGN)
#define DMA_PTR_ALIGN(p) ((typeof(p)) (__ALIGN_KERNEL((uintptr_t) (p), ALIGN_BYTES)))
#define DMA_LEN_ALIGN(n) (((((n) - 1) >> ALIGN_SHIFT) + 1) << ALIGN_SHIFT)
#define MACADDR_BYTES 6

static inline uint8_t ioread8(const volatile unsigned long __iomem addr)
{
	u8 val;

	asm volatile("lb %0, 0(%1)" : "=r" (val) : "r" (addr));
	return val;
}

static inline uint16_t ioread16(const volatile unsigned long __iomem addr)
{
	u16 val;

	asm volatile("lh %0, 0(%1)" : "=r" (val) : "r" (addr));
	return val;
}

static inline uint32_t ioread32(const volatile unsigned long __iomem addr)
{
	u32 val;

	asm volatile("lw %0, 0(%1)" : "=r" (val) : "r" (addr));
	return val;
}

static inline uint64_t ioread64(const volatile unsigned long __iomem addr)
{
	u64 val;

	asm volatile("ld %0, 0(%1)" : "=r" (val) : "r" (addr));
	return val;
}



static inline void iowrite8(uint8_t val, volatile unsigned long __iomem addr)
{
	asm volatile("sb %0, 0(%1)" : : "r" (val), "r" (addr));
}

static inline void iowrite16(uint16_t val, volatile unsigned long __iomem addr)
{
	asm volatile("sh %0, 0(%1)" : : "r" (val), "r" (addr));
}

static inline void iowrite32(uint32_t val, volatile unsigned long __iomem addr)
{
	asm volatile("sw %0, 0(%1)" : : "r" (val), "r" (addr));
}

static inline void iowrite64(uint64_t val, volatile unsigned long __iomem addr)
{
	asm volatile("sd %0, 0(%1)" : : "r" (val), "r" (addr));
}

unsigned long icenet_io_base;
unsigned long icenet_mem_rx_base;
unsigned long icenet_mem_tx_base;
unsigned long rx_packets = 0, rx_bytes = 0;
unsigned long tx_packets = 0, tx_bytes = 0;

#define VIRTIO_BUF_LEN 2048
volatile unsigned long virtio_head = 0;
volatile unsigned long virtio_tail = 0;
struct pkt_buf* virtio_bufs_list[VIRTIO_BUF_LEN];

static inline void virtio_bufs_push(struct pkt_buf *buf)
{
	virtio_bufs_list[virtio_head % VIRTIO_BUF_LEN] = buf;
	virtio_head++;
}

static inline struct pkt_buf* virtio_bufs_pop()
{
	struct pkt_buf *buf;
	buf = virtio_bufs_list[virtio_tail % VIRTIO_BUF_LEN];
	virtio_tail++;
	return buf;
}

#define RECV_BUF_LEN 128
int recv_head = 0;
int recv_tail = 0;
struct pkt_buf* recv_bufs[RECV_BUF_LEN];
struct pkt_buf* recv_bufs_list[RECV_BUF_LEN];

#define SEND_BUF_LEN 128
int send_head = 0;
int send_tail = 0;
struct pkt_buf* send_bufs_list[SEND_BUF_LEN];

static inline void recv_bufs_push(struct pkt_buf *buf)
{
	recv_bufs_list[recv_head] = buf;
	recv_head = (recv_head + 1) & (RECV_BUF_LEN - 1);
}

static inline struct pkt_buf* recv_bufs_pop()
{
	struct pkt_buf *buf;
	buf = recv_bufs_list[recv_tail];
	recv_tail = (recv_tail + 1) & (RECV_BUF_LEN - 1);
	return buf;
}

static inline void send_bufs_push(struct pkt_buf *buf)
{
	send_bufs_list[send_head] = buf;
	send_head = (send_head + 1) & (SEND_BUF_LEN - 1);
}


static inline struct pkt_buf* send_bufs_pop()
{
	struct pkt_buf *buf;
	buf = send_bufs_list[send_tail];
	send_tail = (send_tail + 1) & (SEND_BUF_LEN - 1);
	return buf;
}

static inline int recv_req_avail(void ) {
	return (ioread32(icenet_io_base + ICENET_COUNTS) >> 8) & 0xff;
}

static inline int recv_comp_avail() {
	return (ioread32(icenet_io_base + ICENET_COUNTS) >> 24) & 0xff;
}

static inline int recv_comp_len() {
	return ioread16(icenet_io_base + ICENET_RECV_COMP);
}

static inline int send_req_avail() {
    return ioread32(icenet_io_base + ICENET_COUNTS) & 0xff;
}

static inline int send_space(int nfrags) {
    return send_req_avail() >= nfrags;
}

void alloc_recv(struct mempool *mempool) {
	int recv_cnt = recv_req_avail();
	pkt_buf_alloc_batch(mempool, recv_bufs, recv_cnt);
	
	for(int i = 0; i < recv_cnt; i++) {
        	uint64_t addr = recv_bufs[i]->buf_addr_phy + offsetof(struct pkt_buf, data);
		iowrite64(addr, icenet_io_base + ICENET_RECV_REQ);
		recv_bufs_push(recv_bufs[i]);
	}
}


int icenet_rx_batch_busy(struct pkt_buf* bufs[]){
    if (virtio_tail >= virtio_head)
        return 0;
    if (virtio_head >= virtio_tail + VIRTIO_BUF_LEN){
        printf("%s:%d overflow !!!!\n", __func__, __LINE__);
        return 0;
    }
    bufs[0] = virtio_bufs_pop();

    return 1;
}

void icenet_reclaim_buffer(struct pkt_buf* buf) {
    pkt_buf_free(buf);
}

uint32_t icenet_rx_batch(struct mempool *mempool) {
		int n = recv_comp_avail();
		for (int i = 0; i < n; i++) {
			uint64_t len = ioread16(icenet_io_base + ICENET_RECV_COMP);
			struct pkt_buf* buf = recv_bufs_pop();
            buf->size = len;

			uint8_t csum_res = ioread8(icenet_io_base + ICENET_RXCSUM_RES);
			if (csum_res == 1)
				printf("IIIIIceNet: Checksum offload detected incorrect checksum\n");
            virtio_bufs_push(buf);
		}
        alloc_recv(mempool);
        return n;
}

static inline int send_comp_avail()
{
    return (ioread32(icenet_io_base + ICENET_COUNTS) >> 16) & 0xff;
}

uint32_t icenet_tx_batch(struct virtio_device_userspace* dev,
        uint16_t queue_id, struct pkt_buf* bufs[], uint32_t num_bufs) {
    uint64_t addr;
    uint64_t packet;
    uint64_t partial = 0;
    uint64_t len;
    if(num_bufs > 1) {
        printf("not supported right now!\n");
        return 0;
    }
    for (uint32_t i = 0; i < num_bufs; i++) {
        addr = bufs[i]->buf_addr_phy + offsetof(struct pkt_buf, data);

        len = bufs[i]->size; 
		struct virtio_net_hdr_mrg_rxbuf* hdr = 
            (struct virtio_net_hdr_mrg_rxbuf*) (bufs[i]->data - sizeof(*hdr));
        if (hdr->hdr.flags == VIRTIO_NET_HDR_F_NEEDS_CSUM) {
		    int64_t start, offset, csum_req;
		    start = hdr->hdr.csum_start + 2;
		    offset = start + hdr->hdr.csum_offset;
		    csum_req = (1L << 48) | (offset << 32) | (start << 16);
		    iowrite64(csum_req, icenet_io_base + ICENET_TXCSUM_REQ);
        } else {
		    iowrite64(0L, icenet_io_base + ICENET_TXCSUM_REQ);
        }

        packet = (partial << 63) | (len << 48) | (addr & 0xffffffffffffL);
        send_bufs_push(bufs[i]);
        iowrite64(packet, icenet_io_base + ICENET_SEND_REQ);
    }

    // ack sent buf
    uint16_t send_comp_avails = send_comp_avail();
    for(int send_comp = 0; send_comp < send_comp_avails; send_comp++) {
        int send_comp_ret = ioread16(icenet_io_base + ICENET_SEND_COMP);
        struct pkt_buf* buf = send_bufs_pop();
        pkt_buf_free(buf);
    }
    return num_bufs;
}


static void icenet_init_mac_address()
{
    uint64_t macaddr = ioread64(icenet_io_base + ICENET_MACADDR);
    printf("mac addr is 0x%lx\n", macaddr);
}

// read stat counters and accumulate in stats
// stats may be NULL to just reset the counters
// this is not thread-safe, (but we only support one queue anyways)
// a proper thread-safe implementation would collect per-queue stats
// and perform a read with relaxed memory ordering here without resetting the stats
void icenet_read_stats(struct virtio_device_userspace* dev, struct device_stats* stats) {
	if (stats) {
		stats->rx_pkts += dev->rx_pkts;
		stats->tx_pkts += dev->tx_pkts;
		stats->rx_bytes += dev->rx_bytes;
		stats->tx_bytes += dev->tx_bytes;
	}
	dev->rx_pkts = dev->tx_pkts = dev->rx_bytes = dev->tx_bytes = 0;
}

struct virtio_device_userspace* icenet_init_userspace(const char* name,
        uint16_t rx_queues, uint16_t tx_queues) {
    icenet_io_base = 0x3000008000UL;
    // not build icenet driver, so no need to remove virtio driver
    // remove_virtio_driver(name);
    printf("start icenet init\n");
    struct virtio_device_userspace* dev = calloc(1, sizeof(*dev));
    printf("try to get mac addr\n");
    icenet_init_mac_address();

    dev->read_stats = icenet_read_stats;
//    dev->rx_batch = icenet_rx_batch;
    dev->tx_batch = icenet_tx_batch;
    printf("enable checksum.\n");
    iowrite8(1, icenet_io_base + ICENET_CSUM_ENABLE);
    mb();
    printf("ICENET INIT OK!!!!\n");
    return dev;
}
