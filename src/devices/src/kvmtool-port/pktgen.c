// number of packets sent simultaneously to our driver
static const uint32_t BATCH_SIZE = 64;

// excluding CRC (offloaded by default)
#define PKT_SIZE 60

static const uint8_t pkt_data[] = {
	/* Ethernet */
	0x01, 0x02, 0x03, 0x04, 0x05, 0x06, // dst MAC
	0x10, 0x10, 0x10, 0x10, 0x10, 0x10, // src MAC
	0x08, 0x00,                         // ether type: IPv4
	/* IP */
	0x45, 0x00,                         // Version, IHL, TOS
	(PKT_SIZE - 14) >> 8,               // ip len excluding ethernet, high byte
	(PKT_SIZE - 14) & 0xFF,             // ip len exlucding ethernet, low byte
	0x00, 0x00, 0x00, 0x00,             // id, flags, fragmentation
	0x40, 0x11, 0x00, 0x00,             // TTL (64), protocol (UDP), checksum
	0x0A, 0x00, 0x00, 0x01,             // src ip (10.0.0.1)
	0x0A, 0x00, 0x00, 0x02,             // dst ip (10.0.0.2)
	0x00, 0x2A, 0x05, 0x39,             // src and dst ports (42 -> 1337)
	(PKT_SIZE - 20 - 14) >> 8,          // udp len excluding ip & ethernet, high byte
	(PKT_SIZE - 20 - 14) & 0xFF,        // udp len exlucding ip & ethernet, low byte
	0x00, 0x00,                         // udp checksum, optional
	'i', 'x', 'y'                       // payload
	// rest of the payload is zero-filled because mempools guarantee empty bufs
};

// calculate a IP/TCP/UDP checksum
static uint16_t calc_ip_checksum(uint8_t* data, uint32_t len) {
	if (len % 1) error("odd-sized checksums NYI"); // we don't need that
	uint32_t cs = 0;
	for (uint32_t i = 0; i < len / 2; i++) {
		cs += ((uint16_t*)data)[i];
		if (cs > 0xFFFF) {
			cs = (cs & 0xFFFF) + 1; // 16 bit one's complement
		}
	}
	return ~((uint16_t) cs);
}

static struct mempool* init_mempool() {
	const int NUM_BUFS = 4096;
	struct mempool* mempool = memory_allocate_mempool(NUM_BUFS, 2048);
	// pre-fill all our packet buffers with some templates that can be modified later
	// we have to do it like this because sending is async in the hardware; we cannot re-use a buffer immediately
//	struct pkt_buf* bufs[NUM_BUFS];
//	for (int buf_id = 0; buf_id < NUM_BUFS; buf_id++) {
//		struct pkt_buf* buf = pkt_buf_alloc(mempool);
//		buf->size = PKT_SIZE;
//		memcpy(buf->data, pkt_data, sizeof(pkt_data));
//		*(uint16_t*) (buf->data + 24) = calc_ip_checksum(buf->data + 14, 20);
//		bufs[buf_id] = buf;
//	}
//	// return them all to the mempool, all future allocations will return bufs with the data set above
//	for (int buf_id = 0; buf_id < NUM_BUFS; buf_id++) {
//		pkt_buf_free(bufs[buf_id]);
//	}

	return mempool;
}

static inline uint32_t ixy_rx_batch(struct virtio_device_userspace* dev, uint16_t queue_id, struct pkt_buf* bufs[], uint32_t num_bufs) {
 	return dev->rx_batch(dev, queue_id, bufs, num_bufs);
}
		
static inline void ixy_tx_batch_busy_wait(struct virtio_device_userspace *dev, uint16_t qid,
        struct pkt_buf *bufs[], uint32_t num_bufs) {
    uint32_t num_sent = 0;
    while ((num_sent += dev->tx_batch(dev, qid,
                    bufs + num_sent, num_bufs - num_sent)) != num_bufs) {
        // busy wait
    }
}

static inline void ixy_read_stats(struct virtio_device_userspace *dev,
        struct device_stats *stats) {
    dev->read_stats(dev, stats);
}

int pktgen(struct virtio_device_userspace* dev) {
	struct mempool* mempool = init_mempool();

	uint64_t last_stats_printed = monotonic_time();
	uint64_t counter = 0;
	struct device_stats stats_old, stats;
	stats_init(&stats, dev);
	stats_init(&stats_old, dev);
	uint32_t seq_num = 0;

	// array of bufs sent out in a batch
	struct pkt_buf* send_bufs[BATCH_SIZE];
	struct pkt_buf* recv_bufs[BATCH_SIZE];

	// tx loop
	while (true) {
		uint32_t num_rx = ixy_rx_batch(dev, 0, recv_bufs, BATCH_SIZE);
		for (uint32_t i = 0; i < num_rx; i++) {

			pkt_buf_free(recv_bufs[i]);
		}
		// we cannot immediately recycle packets, we need to allocate new packets every time
		// the old packets might still be used by the NIC: tx is async
		pkt_buf_alloc_batch(mempool, send_bufs, BATCH_SIZE);
		for (uint32_t i = 0; i < BATCH_SIZE; i++) {
			// packets can be modified here, make sure to update the checksum when changing the IP header
			*(uint32_t*)(send_bufs[i]->data + PKT_SIZE - 4) = seq_num++;
		}
		// the packets could be modified here to generate multiple flows
		ixy_tx_batch_busy_wait(dev, 0, send_bufs, BATCH_SIZE);

		// don't check time for every packet, this yields +10% performance :)
		if ((counter++ & 0xFFF) == 0) {
			uint64_t time = monotonic_time();
			if (time - last_stats_printed > 1000 * 1000 * 1000) {
				// every second
				ixy_read_stats(dev, &stats);
				print_stats_diff(&stats, &stats_old, time - last_stats_printed);
				stats_old = stats;
				last_stats_printed = time;
			}
		}
		// track stats
	}
}

