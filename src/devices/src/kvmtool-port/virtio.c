struct virtio_device_userspace {
	void *mmio_addr;
    uint64_t mmio_size;
    
    uint16_t num_rx_queues;
    void* rx_queue;
    uint64_t rx_pkts;
    uint64_t rx_bytes;
	
    uint16_t num_tx_queues;
	void* tx_queue;
	uint64_t tx_pkts;
	uint64_t tx_bytes;
	
    void* ctrl_queue;
    
    void (*read_stats) (struct virtio_device_userspace *dev, struct device_stats *stat);
    void (*set_promisc) (struct virtio_device_userspace *dev, bool enabled);
    uint32_t (*get_link_speed) (const struct virtio_device_userspace *dev);
    uint32_t (*rx_batch) (struct virtio_device_userspace *dev, uint16_t qid,
            struct pkt_buf *bufs[], uint32_t num_bufs);
    uint32_t (*tx_batch) (struct virtio_device_userspace *dev, uint16_t qid,
            struct pkt_buf *bufs[], uint32_t num_bufs);
};

static inline size_t virtio_legacy_vring_size(unsigned int num, unsigned long align) {
	size_t size;

	size = num * sizeof(struct vring_desc);
	size += sizeof(struct vring_avail) + (num * sizeof(uint16_t));
	size = RTE_ALIGN_CEIL(size, align);
	size += sizeof(struct vring_used) + (num * sizeof(struct vring_used_elem));
	return size;
}

static inline void virtio_legacy_vring_init(struct vring *vr,
        unsigned int num, uint8_t *p, unsigned long align) {
	vr->num = num;
	vr->desc = (struct vring_desc *)p;
	vr->avail = (struct vring_avail *)(p + num * sizeof(struct vring_desc));
	vr->used = (void *)RTE_ALIGN_CEIL((uintptr_t)(&vr->avail->ring[num]), align);
}

static const struct virtio_legacy_net_hdr net_hdr = {
	.flags = 0,
	.gso_type = VIRTIO_NET_HDR_GSO_NONE,
	.hdr_len = 14 + 20 + 8,
};

static void virtio_legacy_setup_queue(struct virtio_device_userspace *dev, uint16_t qid) {
    /* Create a virtual queue - Section 2.4.1 */
    writel(qid, dev->mmio_addr + VIRTIO_MMIO_QUEUE_SEL);
    uint32_t max_queue_num = readl(dev->mmio_addr + VIRTIO_MMIO_QUEUE_NUM_MAX);
    debug("qid: %x, max queue num: %x", qid, max_queue_num);
    if (max_queue_num == 0)
        return;

    uint32_t queue_num = 0x400; // 1024
    uint32_t queue_align = 0x1000; // 4096
    writel(queue_num, dev->mmio_addr + VIRTIO_MMIO_QUEUE_NUM);
    writel(queue_align, dev->mmio_addr + VIRTIO_MMIO_QUEUE_ALIGN);

    size_t queue_mem_size = virtio_legacy_vring_size(queue_num, queue_align);
    struct dma_memory mem = memory_allocate_dma(queue_mem_size, true);
    memset(mem.virt, 0xab, queue_mem_size);
    debug("allocated %zu bytes for qid %x at %p, pa %lx, mem %x",
            queue_mem_size, qid, mem.virt, mem.phy, *(int *)mem.virt);
    writel(mem.phy >> PAGE_SHIFT, dev->mmio_addr + VIRTIO_MMIO_QUEUE_PFN);

    /* For layout - Section 2.4.2 */
	struct virtqueue* vq = calloc(1, sizeof(*vq) + sizeof(void*) * queue_num);
    virtio_legacy_vring_init(&vq->vring, queue_num, mem.virt, queue_align);
	debug("vring desc: %p, vring avail: %p, vring used: %p", vq->vring.desc, vq->vring.avail, vq->vring.used);
	for (size_t i = 0; i < vq->vring.num; ++i) {
		vq->vring.desc[i].len = 0;
		vq->vring.desc[i].addr = 0;
		vq->vring.desc[i].flags = 0;
		vq->vring.desc[i].next = 0;
		vq->vring.avail->ring[i] = 0;
		vq->vring.used->ring[i].id = 0;
		vq->vring.used->ring[i].len = 0;
	}
	vq->vring.used->idx = 0;
	vq->vring.avail->idx = 0;
	vq->vq_used_last_idx = 0;

	/* Disable interrupts - Section 2.4.7 */
	vq->vring.avail->flags = VRING_AVAIL_F_NO_INTERRUPT;
	vq->vring.used->flags = 0;

    switch (qid) {
        case 0: { // RX
	        /* Allocate buffers and fill descriptor table - Section 3.2.1
	         * We allocate more bufs than what would fit in the queue,
	         * because we don't want to stall rx if users hold bufs for longer
             */
            /* FIXME: why queue_num * 4? */
	        vq->mempool = memory_allocate_mempool(queue_num * 4, 2048);
	        dev->rx_queue = vq;
            break;
        }
        case 1: { // TX
	        dev->tx_queue = vq;
            break;
        }
        case 2: { // CTRL
            /* FIXME: why 2048? beacuse 2048 > 1500? */
	        vq->mempool = memory_allocate_mempool(queue_num, 2048);
	        dev->ctrl_queue = vq;
            break;
        }
        default: {
            error("invalid qid: %x", qid);
            break;
        }
    }
}

static inline void virtio_legacy_notify_queue(struct virtio_device_userspace* dev, uint16_t qid) {
	writel(qid, dev->mmio_addr + VIRTIO_MMIO_QUEUE_NOTIFY);
}

static void virtio_legacy_check_status(struct virtio_device_userspace *dev) {
    if (readl(dev->mmio_addr + VIRTIO_MMIO_STATUS) == VIRTIO_CONFIG_S_FAILED) {
        error("device status VIRTIO_CONFIG_S_FAILED");
    }
    if (readl(dev->mmio_addr + VIRTIO_MMIO_STATUS) == VIRTIO_CONFIG_S_NEEDS_RESET) {
        error("device status VIRTIO_CONFIG_S_NEEDS_RESET");
    }
}

static void virtio_legacy_send_command(struct virtio_device_userspace *dev,
        void *cmd, size_t cmd_len) {
	struct virtqueue *vq = dev->ctrl_queue;

	if (cmd_len < sizeof(struct virtio_net_ctrl_hdr)) {
		error("Command can not be shorter than control header");
	}
	if (((uint8_t *)cmd)[0] != VIRTIO_NET_CTRL_RX) {
		error("Command class is not supported");
	}

	_mm_mfence();
	// Find free desciptor slot
	uint16_t idx = 0;
	for (idx = 0; idx < vq->vring.num; ++idx) {
		struct vring_desc *desc = &vq->vring.desc[idx];
		if (desc->addr == 0) {
			break;
		}
	}
	if (idx == vq->vring.num) {
		error("command queue full");
	} else {
		debug("Found free desc slot at %u (%u)", idx, vq->vring.num);
	}

	struct pkt_buf *buf = pkt_buf_alloc(vq->mempool);
	if (!buf) {
		error("Control queue ran out of buffers");
	}
    debug("buf->data: %p", buf->data);
	memcpy(buf->data, cmd, cmd_len);
	vq->virtual_addresses[idx] = buf;

	/* The following descriptor setup kills QEMU, but should be allowed with VIRTIO_F_ANY_LAYOUT
	 * Error: kvm: virtio-net ctrl missing headers
	 * Version: QEMU emulator version 2.7.1 pve-qemu-kvm_2.7.1-4
	 */
	// All in one descriptor
	// vq->vring.desc[idx].len = cmd_len;
	// vq->vring.desc[idx].addr = buf->buf_addr_phy + offsetof(struct pkt_buf, data);
	// vq->vring.desc[idx].flags = VRING_DESC_F_WRITE;
	// vq->vring.desc[idx].next = 0;

	// Device-readable head: cmd header
	vq->vring.desc[idx].len = 2;
	vq->vring.desc[idx].addr = buf->buf_addr_phy + offsetof(struct pkt_buf, data);
	vq->vring.desc[idx].flags = VRING_DESC_F_NEXT;
	vq->vring.desc[idx].next = idx + 1;
	// Device-readable payload: data
	vq->vring.desc[idx + 1].len = cmd_len - 2 - 1; // Header and ack byte
	vq->vring.desc[idx + 1].addr = buf->buf_addr_phy + offsetof(struct pkt_buf, data) + 2;
	vq->vring.desc[idx + 1].flags = VRING_DESC_F_NEXT;
	vq->vring.desc[idx + 1].next = idx + 2;
	// Device-writable tail: ack flag
	vq->vring.desc[idx + 2].len = 1;
	vq->vring.desc[idx + 2].addr = buf->buf_addr_phy + offsetof(struct pkt_buf, data) + cmd_len - 1;
	vq->vring.desc[idx + 2].flags = VRING_DESC_F_WRITE;
	vq->vring.desc[idx + 2].next = 0;
	vq->vring.avail->ring[vq->vring.avail->idx % vq->vring.num] = idx;
	_mm_mfence();
	vq->vring.avail->idx++;
	_mm_mfence();

	virtio_legacy_notify_queue(dev, 2);
	_mm_mfence();

	// Wait until the buffer got processed
	while (vq->vq_used_last_idx == vq->vring.used->idx) {
		_mm_mfence();
		debug("Waiting...");
		usleep(100000);
	}
	vq->vq_used_last_idx++;
	// Check status and free buffer
	struct vring_used_elem *e = &vq->vring.used->ring[vq->vring.used->idx];
	debug("e %p: id %u len %u", e, e->id, e->len);
	if (e->id != idx) {
		error("Used buffer has different index as sent one");
	}
	if (vq->virtual_addresses[idx] != buf) {
		error("buffer differ");
	}
	pkt_buf_free(buf);
	vq->vring.desc[idx] = (struct vring_desc){};
	vq->vring.desc[idx + 1] = (struct vring_desc){};
	vq->vring.desc[idx + 2] = (struct vring_desc){};
}

static void virtio_legacy_set_promiscuous(struct virtio_device_userspace *dev, bool on) {
	struct {
		struct virtio_net_ctrl_hdr hdr;
		uint8_t on;
		uint8_t ack;
	} __attribute__((__packed__)) cmd = {};
	static_assert(sizeof(cmd) == 4, "Size of command struct wrong");

	cmd.hdr.class = VIRTIO_NET_CTRL_RX;
	cmd.hdr.cmd = VIRTIO_NET_CTRL_RX_PROMISC;
	cmd.on = on ? 1 : 0;

	virtio_legacy_send_command(dev, &cmd, sizeof(cmd));
	info("Set promisc to %u", on);
}

static void virtio_legacy_init(struct virtio_device_userspace *dev) {
    /*
     * ACK: Indicates that the guest OS has found the device and recognized it 
     * as a valid virtio device.
     * DRIVER: Indicates that the guest OS knows how to drive the device.
     */
    debug("val: %x, addr: %p", 
            VIRTIO_CONFIG_S_ACKNOWLEDGE | VIRTIO_CONFIG_S_DRIVER,
            dev->mmio_addr + VIRTIO_MMIO_STATUS);
    writel(VIRTIO_CONFIG_S_ACKNOWLEDGE | VIRTIO_CONFIG_S_DRIVER,
            dev->mmio_addr + VIRTIO_MMIO_STATUS);

    /* High 32 bits */
    writel(1, dev->mmio_addr + VIRTIO_MMIO_DEVICE_FEATURES_SEL);
    writel(0, dev->mmio_addr + VIRTIO_MMIO_DEVICE_FEATURES);
    /* Low 32 bits */
    writel(0, dev->mmio_addr + VIRTIO_MMIO_DEVICE_FEATURES_SEL);
    uint32_t dev_features = readl(dev->mmio_addr + VIRTIO_MMIO_DEVICE_FEATURES);
    uint32_t driver_features = (1u << VIRTIO_F_ANY_LAYOUT)
        | (1u << VIRTIO_NET_F_CSUM) | (1u << VIRTIO_NET_F_GUEST_CSUM)
        | (1u << VIRTIO_NET_F_CTRL_VQ) | (1u << VIRTIO_NET_F_CTRL_RX);
    if ((dev_features & driver_features) != driver_features) {
        error("device %x does NOT support driver features %x",
                dev_features, driver_features);
    }
    writel(driver_features, dev->mmio_addr + VIRTIO_MMIO_DEVICE_FEATURES);

    /* RX */
    virtio_legacy_setup_queue(dev, 0);
    /* TX */
    virtio_legacy_setup_queue(dev, 1);
    /* Control */
    virtio_legacy_setup_queue(dev, 2);
    
    writel(VIRTIO_CONFIG_S_ACKNOWLEDGE | VIRTIO_CONFIG_S_DRIVER
            | VIRTIO_CONFIG_S_DRIVER_OK, dev->mmio_addr + VIRTIO_MMIO_STATUS);
    debug("setup complete");

    /* Re-check status */
    virtio_legacy_check_status(dev);
    /* Enable promiscuous */
    virtio_legacy_set_promiscuous(dev, true);
}

// read stat counters and accumulate in stats
// stats may be NULL to just reset the counters
// this is not thread-safe, (but we only support one queue anyways)
// a proper thread-safe implementation would collect per-queue stats
// and perform a read with relaxed memory ordering here without resetting the stats
void virtio_read_stats(struct virtio_device_userspace* dev, struct device_stats* stats) {
	if (stats) {
		stats->rx_pkts += dev->rx_pkts;
		stats->tx_pkts += dev->tx_pkts;
		stats->rx_bytes += dev->rx_bytes;
		stats->tx_bytes += dev->tx_bytes;
	}
	dev->rx_pkts = dev->tx_pkts = dev->rx_bytes = dev->tx_bytes = 0;
}

void virtio_set_promisc(struct virtio_device_userspace* dev, bool enabled) {
    virtio_legacy_set_promiscuous(dev, enabled);
}

uint32_t virtio_get_link_speed(const struct virtio_device_userspace* dev) {
	return 1000;
}

uint32_t virtio_rx_batch(struct virtio_device_userspace* dev,
        uint16_t queue_id, struct pkt_buf* bufs[], uint32_t num_bufs) {
	struct virtqueue* vq = dev->rx_queue;
	uint32_t buf_idx;

	_mm_mfence();
	// Retrieve used bufs from the device
	for (buf_idx = 0; buf_idx < num_bufs; ++buf_idx) {
		// Section 3.2.2
		if (vq->vq_used_last_idx == vq->vring.used->idx) {
			break;
		}
		// info("Rx packet: last used %u, used idx %u", vq->vq_used_last_idx,
		// vq->vring.used->idx);
		struct vring_used_elem* e = vq->vring.used->ring + (vq->vq_used_last_idx % vq->vring.num);
		// info("Used elem %p, id %u len %u", e, e->id, e->len);
		struct vring_desc* desc = &vq->vring.desc[e->id];
		vq->vq_used_last_idx++;
		// We don't support chaining or indirect descriptors
		if (desc->flags != VRING_DESC_F_WRITE) {
			error("unsupported rx flags on descriptor: %x", desc->flags);
		}
		// info("Desc %lu %u %u %u", desc->addr, desc->len, desc->flags,
		// desc->next);
		*desc = (struct vring_desc){};

		// Section 5.1.6.4
		struct pkt_buf* buf = vq->virtual_addresses[e->id];
		buf->size = e->len - sizeof(net_hdr);
		bufs[buf_idx] = buf;
		//struct virtio_net_hdr* hdr = (void*)(buf->head_room + sizeof(buf->head_room) - sizeof(net_hdr));

		// Update rx counter
		dev->rx_bytes += buf->size;
		dev->rx_pkts++;
	}
	// Fill empty slots in descriptor table
	for (uint16_t idx = 0; idx < vq->vring.num; ++idx) {
		struct vring_desc* desc = &vq->vring.desc[idx];
		if (desc->addr != 0) { // descriptor points to something, therefore it is in use
			continue;
		}
		// info("Found free desc slot at %u (%u)", idx, vq->vring.num);
		struct pkt_buf* buf = pkt_buf_alloc(vq->mempool);
		if (!buf) {
			error("failed to allocate new mbuf for rx, you are either leaking memory or your mempool is too small");
		}
		buf->size = vq->mempool->buf_size;
		memcpy(buf->head_room + sizeof(buf->head_room) - sizeof(net_hdr), &net_hdr, sizeof(net_hdr));
		vq->vring.desc[idx].len = buf->size + sizeof(net_hdr);
		vq->vring.desc[idx].addr =
			buf->buf_addr_phy + offsetof(struct pkt_buf, head_room) + sizeof(buf->head_room) - sizeof(net_hdr);
		vq->vring.desc[idx].flags = VRING_DESC_F_WRITE;
		vq->vring.desc[idx].next = 0;
		vq->virtual_addresses[idx] = buf;
		vq->vring.avail->ring[vq->vring.avail->idx % vq->vring.num] = idx;
		_mm_mfence(); // Make sure exposed descriptors reach device before index is updated
		vq->vring.avail->idx++;
		_mm_mfence(); // Make sure the index update reaches device before it is triggered
		virtio_legacy_notify_queue(dev, 0);
	}
	return buf_idx;
}

uint32_t virtio_tx_batch(struct virtio_device_userspace* dev,
        uint16_t queue_id, struct pkt_buf* bufs[], uint32_t num_bufs) {
	struct virtqueue* vq = dev->tx_queue;

	_mm_mfence();
	// Free sent buffers
	while (vq->vq_used_last_idx != vq->vring.used->idx) {
		// info("We can free some buffers: %u != %u", vq->vq_used_last_idx,
		// vq->vring.used->idx);
		struct vring_used_elem* e = vq->vring.used->ring + (vq->vq_used_last_idx % vq->vring.num);
		// info("e %p, id %u", e, e->id);
		struct vring_desc* desc = &vq->vring.desc[e->id];
		desc->addr = 0;
		desc->len = 0;
		pkt_buf_free(vq->virtual_addresses[e->id]);
		vq->vq_used_last_idx++;
		_mm_mfence();
	}
	// Send buffers
	uint32_t buf_idx;
	uint16_t idx = 0; // Keep index of last found free descriptor and start searching from there
	for (buf_idx = 0; buf_idx < num_bufs; ++buf_idx) {
		struct pkt_buf* buf = bufs[buf_idx];
		// Find free desc index
		for (; idx < vq->vring.num; ++idx) {
			struct vring_desc* desc = &vq->vring.desc[idx];
			if (desc->addr == 0) {
				break;
			}
		}
		if (idx == vq->vring.num) {
			break;
		}
		// info("Found free desc slot at %u (%u)", idx, vq->vring.num);

		// Update tx counter
		dev->tx_bytes += buf->size;
		dev->tx_pkts++;

		vq->virtual_addresses[idx] = buf;

		// Copy header to headroom in front of data buffer
		// memcpy(buf->head_room + sizeof(buf->head_room) - sizeof(net_hdr), &net_hdr, sizeof(net_hdr));

		vq->vring.desc[idx].len = buf->size + sizeof(net_hdr);
		vq->vring.desc[idx].addr =
			buf->buf_addr_phy + offsetof(struct pkt_buf, head_room) + sizeof(buf->head_room) - sizeof(net_hdr);
		vq->vring.desc[idx].flags = 0;
		vq->vring.desc[idx].next = 0;
		vq->vring.avail->ring[(vq->vring.avail->idx + buf_idx) % vq->vring.num] = idx;
	}
	_mm_mfence();
	vq->vring.avail->idx += buf_idx;
	_mm_mfence();
	virtio_legacy_notify_queue(dev, 1);
	return buf_idx;
}

struct virtio_device_userspace* virtio_init_userspace(const char* name,
        uint16_t rx_queues, uint16_t tx_queues) {
    struct virtio_device_userspace* dev = calloc(1, sizeof(*dev));
    dev->mmio_addr = (void *)0x3000008000UL;
    dev->mmio_size = 0x1000;
    dev->num_rx_queues = rx_queues;
    dev->num_tx_queues = tx_queues;
	
	dev->read_stats = virtio_read_stats;
	dev->set_promisc = virtio_set_promisc;
	dev->get_link_speed = virtio_get_link_speed;
    dev->rx_batch = virtio_rx_batch;
	dev->tx_batch = virtio_tx_batch;
	
    virtio_legacy_init(dev);

	return dev;
}
