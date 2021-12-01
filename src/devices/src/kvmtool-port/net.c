#include "kvm/virtio-pci-dev.h"
#include "kvm/virtio-net.h"
#include "kvm/virtio.h"
#include "kvm/mutex.h"
#include "kvm/util.h"
#include "kvm/kvm.h"
#include "kvm/irq.h"
#include "kvm/uip.h"
#include "kvm/guest_compat.h"
#include "kvm/iovec.h"
#include "kvm/strbuf.h"

#include <linux/vhost.h>
#include <linux/virtio_net.h>
#include <linux/if_tun.h>
#include <linux/types.h>

#include <arpa/inet.h>
#include <net/if.h>

#include <unistd.h>
#include <fcntl.h>

#include <sys/socket.h>
#include <sys/ioctl.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <sys/eventfd.h>

#define VIRTIO_NET_QUEUE_SIZE		256
#define VIRTIO_NET_NUM_QUEUES		8

struct net_dev;

struct net_dev_operations {
	int (*rx)(struct iovec *iov, u16 in, struct net_dev *ndev);
	int (*tx)(struct iovec *iov, u16 in, struct net_dev *ndev);
};

struct net_dev_queue {
	int				id;
	struct net_dev			*ndev;
	struct virt_queue		vq;
	pthread_t			thread;
	struct mutex			lock;
	pthread_cond_t			cond;
	int				gsi;
	int				irqfd;
};

struct net_dev {
	struct mutex			mutex;
	struct virtio_device		vdev;
	struct list_head		list;

	struct net_dev_queue		queues[VIRTIO_NET_NUM_QUEUES * 2 + 1];
	struct virtio_net_config	config;
	u32				features, queue_pairs;

	int				vhost_fd;
	int				tap_fd;
	char				tap_name[IFNAMSIZ];
	bool				tap_ufo;

	int				mode;

	struct uip_info			info;
	struct net_dev_operations	*ops;
	struct kvm			*kvm;

	struct virtio_net_params	*params;
};

static LIST_HEAD(ndevs);

#define MAX_PACKET_SIZE 65550

static bool has_virtio_feature(struct net_dev *ndev, u32 feature)
{
	return ndev->features & (1 << feature);
}

static void virtio_net_fix_tx_hdr(struct virtio_net_hdr *hdr, struct net_dev *ndev)
{
	hdr->hdr_len		= virtio_guest_to_host_u16(&ndev->vdev, hdr->hdr_len);
	hdr->gso_size		= virtio_guest_to_host_u16(&ndev->vdev, hdr->gso_size);
	hdr->csum_start		= virtio_guest_to_host_u16(&ndev->vdev, hdr->csum_start);
	hdr->csum_offset	= virtio_guest_to_host_u16(&ndev->vdev, hdr->csum_offset);
}

static void virtio_net_fix_rx_hdr(struct virtio_net_hdr *hdr, struct net_dev *ndev)
{
	hdr->hdr_len		= virtio_host_to_guest_u16(&ndev->vdev, hdr->hdr_len);
	hdr->gso_size		= virtio_host_to_guest_u16(&ndev->vdev, hdr->gso_size);
	hdr->csum_start		= virtio_host_to_guest_u16(&ndev->vdev, hdr->csum_start);
	hdr->csum_offset	= virtio_host_to_guest_u16(&ndev->vdev, hdr->csum_offset);
}

static void *virtio_net_rx_thread(void *p)
{
	struct iovec iov[VIRTIO_NET_QUEUE_SIZE];
	struct net_dev_queue *queue = p;
	struct virt_queue *vq = &queue->vq;
	struct net_dev *ndev = queue->ndev;
	struct kvm *kvm;
	u16 out, in;
	u16 head;
	int len, copied;

	kvm__set_thread_name("virtio-net-rx");
    printf("--- %s:%d desc %p, avail %p, used %p\n", __func__, __LINE__,
            vq->vring.desc, vq->vring.avail, vq->vring.used);
	
    kvm = ndev->kvm;
	while (1) {
		mutex_lock(&queue->lock);
		if (!virt_queue__available(vq))
			pthread_cond_wait(&queue->cond, &queue->lock.mutex);
		mutex_unlock(&queue->lock);

		while (virt_queue__available(vq)) {
			unsigned char buffer[MAX_PACKET_SIZE + sizeof(struct virtio_net_hdr_mrg_rxbuf)];
			struct iovec dummy_iov = {
				.iov_base = buffer,
				.iov_len  = sizeof(buffer),
			};
			struct virtio_net_hdr_mrg_rxbuf *hdr;
			u16 num_buffers;

			len = ndev->ops->rx(&dummy_iov, 1, ndev);
			if (len < 0) {
				pr_warning("%s: rx on vq %u failed (%d), exiting thread\n",
						__func__, queue->id, len);
				goto out_err;
			}

			copied = num_buffers = 0;
			head = virt_queue__get_iov(vq, iov, &out, &in, kvm);
			hdr = iov[0].iov_base;
			while (copied < len) {
				size_t iovsize = min_t(size_t, len - copied, iov_size(iov, in));

				memcpy_toiovec(iov, buffer + copied, iovsize);
				copied += iovsize;
				virt_queue__set_used_elem_no_update(vq, head, iovsize, num_buffers++);
				if (copied == len)
					break;
				while (!virt_queue__available(vq))
					sleep(0);
				head = virt_queue__get_iov(vq, iov, &out, &in, kvm);
			}

			virtio_net_fix_rx_hdr(&hdr->hdr, ndev);
			if (has_virtio_feature(ndev, VIRTIO_NET_F_MRG_RXBUF))
				hdr->num_buffers = virtio_host_to_guest_u16(vq, num_buffers);

			virt_queue__used_idx_advance(vq, num_buffers);

			/* We should interrupt guest right now, otherwise latency is huge. */
			if (virtio_queue__should_signal(vq)) {
				ndev->vdev.ops->signal_vq(kvm, &ndev->vdev, queue->id);
            }
		}
	}

out_err:
	pthread_exit(NULL);
	return NULL;

}

static void *virtio_net_tx_thread(void *p)
{
	struct iovec iov[VIRTIO_NET_QUEUE_SIZE];
	struct net_dev_queue *queue = p;
	struct virt_queue *vq = &queue->vq;
	struct net_dev *ndev = queue->ndev;
	struct kvm *kvm;
	u16 out, in;
	u16 head;
	int len;

	kvm__set_thread_name("virtio-net-tx");
    printf("--- %s:%d desc %p, avail %p, used %p\n", __func__, __LINE__,
            vq->vring.desc, vq->vring.avail, vq->vring.used);

	kvm = ndev->kvm;

	while (1) {
		mutex_lock(&queue->lock);
		if (!virt_queue__available(vq))
			pthread_cond_wait(&queue->cond, &queue->lock.mutex);
		mutex_unlock(&queue->lock);

		while (virt_queue__available(vq)) {
			struct virtio_net_hdr *hdr;
			head = virt_queue__get_iov(vq, iov, &out, &in, kvm);
			hdr = iov[0].iov_base;
			virtio_net_fix_tx_hdr(hdr, ndev);
			len = ndev->ops->tx(iov, out, ndev);
			if (len < 0) {
				pr_warning("%s: tx on vq %u failed (%d)\n",
						__func__, queue->id, errno);
				goto out_err;
			}

			virt_queue__set_used_elem(vq, head, len);
		}

		if (virtio_queue__should_signal(vq))
			ndev->vdev.ops->signal_vq(kvm, &ndev->vdev, queue->id);
	}

out_err:
	pthread_exit(NULL);
	return NULL;
}

static virtio_net_ctrl_ack virtio_net_handle_mq(struct kvm* kvm, struct net_dev *ndev, struct virtio_net_ctrl_hdr *ctrl)
{
	/* Not much to do here */
	return VIRTIO_NET_OK;
}

static void *virtio_net_ctrl_thread(void *p)
{
	struct iovec iov[VIRTIO_NET_QUEUE_SIZE];
	struct net_dev_queue *queue = p;
	struct virt_queue *vq = &queue->vq;
	struct net_dev *ndev = queue->ndev;
	u16 out, in, head;
	struct kvm *kvm = ndev->kvm;
	struct virtio_net_ctrl_hdr *ctrl;
	virtio_net_ctrl_ack *ack;

	kvm__set_thread_name("virtio-net-ctrl");
    printf("--- %s:%d desc %p, avail %p, used %p\n", __func__, __LINE__,
            vq->vring.desc, vq->vring.avail, vq->vring.used);

	while (1) {
		mutex_lock(&queue->lock);
		if (!virt_queue__available(vq))
			pthread_cond_wait(&queue->cond, &queue->lock.mutex);
		mutex_unlock(&queue->lock);

		while (virt_queue__available(vq)) {
			head = virt_queue__get_iov(vq, iov, &out, &in, kvm);
			ctrl = iov[0].iov_base;
			ack = iov[out].iov_base;

			switch (ctrl->class) {
			case VIRTIO_NET_CTRL_MQ:
				*ack = virtio_net_handle_mq(kvm, ndev, ctrl);
				break;
			default:
				*ack = VIRTIO_NET_ERR;
				break;
			}
			virt_queue__set_used_elem(vq, head, iov[out].iov_len);
		}

		if (virtio_queue__should_signal(vq))
			ndev->vdev.ops->signal_vq(kvm, &ndev->vdev, queue->id);
	}

	pthread_exit(NULL);

	return NULL;
}

static void virtio_net_handle_callback(struct kvm *kvm, struct net_dev *ndev, int queue)
{
	struct net_dev_queue *net_queue = &ndev->queues[queue];

	if ((u32)queue >= (ndev->queue_pairs * 2 + 1)) {
		pr_warning("Unknown queue index %u", queue);
		return;
	}

	mutex_lock(&net_queue->lock);
	pthread_cond_signal(&net_queue->cond);
	mutex_unlock(&net_queue->lock);
}

static int virtio_net_request_tap(struct net_dev *ndev, struct ifreq *ifr,
				  const char *tapname)
{
	int ret;

	memset(ifr, 0, sizeof(*ifr));
	ifr->ifr_flags = IFF_TAP | IFF_NO_PI | IFF_VNET_HDR;
	if (tapname)
		strlcpy(ifr->ifr_name, tapname, sizeof(ifr->ifr_name));

	ret = ioctl(ndev->tap_fd, TUNSETIFF, ifr);

	if (ret >= 0)
		strlcpy(ndev->tap_name, ifr->ifr_name, sizeof(ndev->tap_name));
	return ret;
}

static int virtio_net_exec_script(const char* script, const char *tap_name)
{
	pid_t pid;
	int status;

	pid = fork();
	if (pid == 0) {
		execl(script, script, tap_name, NULL);
		_exit(1);
	} else {
		waitpid(pid, &status, 0);
		if (WIFEXITED(status) && WEXITSTATUS(status) != 0) {
			pr_warning("Fail to setup tap by %s", script);
			return -1;
		}
	}
	return 0;
}

static bool virtio_net__tap_init(struct net_dev *ndev)
{
	int sock = socket(AF_INET, SOCK_STREAM, 0);
	int hdr_len;
	struct sockaddr_in sin = {0};
	struct ifreq ifr;
	const struct virtio_net_params *params = ndev->params;
	bool skipconf = !!params->tapif;

	hdr_len = has_virtio_feature(ndev, VIRTIO_NET_F_MRG_RXBUF) ?
			sizeof(struct virtio_net_hdr_mrg_rxbuf) :
			sizeof(struct virtio_net_hdr);
	if (ioctl(ndev->tap_fd, TUNSETVNETHDRSZ, &hdr_len) < 0)
		pr_warning("Config tap device TUNSETVNETHDRSZ error");

	if (strcmp(params->script, "none")) {
		if (virtio_net_exec_script(params->script, ndev->tap_name) < 0)
			goto fail;
	} else if (!skipconf) {
		memset(&ifr, 0, sizeof(ifr));
		strncpy(ifr.ifr_name, ndev->tap_name, sizeof(ifr.ifr_name));
		sin.sin_addr.s_addr = inet_addr(params->host_ip);
		memcpy(&(ifr.ifr_addr), &sin, sizeof(ifr.ifr_addr));
		ifr.ifr_addr.sa_family = AF_INET;
		if (ioctl(sock, SIOCSIFADDR, &ifr) < 0) {
			pr_warning("Could not set ip address on tap device");
			goto fail;
		}
	}

	if (!skipconf) {
		memset(&ifr, 0, sizeof(ifr));
		strncpy(ifr.ifr_name, ndev->tap_name, sizeof(ifr.ifr_name));
		ioctl(sock, SIOCGIFFLAGS, &ifr);
		ifr.ifr_flags |= IFF_UP | IFF_RUNNING;
		if (ioctl(sock, SIOCSIFFLAGS, &ifr) < 0)
			pr_warning("Could not bring tap device up");
	}

	close(sock);

	return 1;

fail:
	if (sock >= 0)
		close(sock);
	if (ndev->tap_fd >= 0)
		close(ndev->tap_fd);

	return 0;
}

static void virtio_net__tap_exit(struct net_dev *ndev)
{
	int sock;
	struct ifreq ifr;

	if (ndev->params->tapif)
		return;

	sock = socket(AF_INET, SOCK_STREAM, 0);
	strncpy(ifr.ifr_name, ndev->tap_name, sizeof(ifr.ifr_name));
	ioctl(sock, SIOCGIFFLAGS, &ifr);
	ifr.ifr_flags &= ~(IFF_UP | IFF_RUNNING);
	if (ioctl(sock, SIOCGIFFLAGS, &ifr) < 0)
		pr_warning("Count not bring tap device down");
	close(sock);
}

static bool virtio_net__tap_create(struct net_dev *ndev)
{
	int offload;
	struct ifreq ifr;
	const struct virtio_net_params *params = ndev->params;
	bool macvtap = (!!params->tapif) && (params->tapif[0] == '/');

	/* Did the user already gave us the FD? */
	if (params->fd)
		ndev->tap_fd = params->fd;
	else {
		const char *tap_file = "/dev/net/tun";

		/* Did the user ask us to use macvtap? */
		if (macvtap)
			tap_file = params->tapif;

		ndev->tap_fd = open(tap_file, O_RDWR);
		if (ndev->tap_fd < 0) {
			pr_warning("Unable to open %s", tap_file);
			return 0;
		}
	}

	if (!macvtap &&
	    virtio_net_request_tap(ndev, &ifr, params->tapif) < 0) {
		pr_warning("Config tap device error. Are you root?");
		goto fail;
	}

	/*
	 * The UFO support had been removed from kernel in commit:
	 * ID: fb652fdfe83710da0ca13448a41b7ed027d0a984
	 * https://www.spinics.net/lists/netdev/msg443562.html
	 * In oder to support the older kernels without this commit,
	 * we set the TUN_F_UFO to offload by default to test the status of
	 * UFO kernel support.
	 */
	ndev->tap_ufo = true;
	offload = TUN_F_CSUM | TUN_F_TSO4 | TUN_F_TSO6 | TUN_F_UFO;
	if (ioctl(ndev->tap_fd, TUNSETOFFLOAD, offload) < 0) {
		/*
		 * Is this failure caused by kernel remove the UFO support?
		 * Try TUNSETOFFLOAD without TUN_F_UFO.
		 */
		offload &= ~TUN_F_UFO;
		if (ioctl(ndev->tap_fd, TUNSETOFFLOAD, offload) < 0) {
			pr_warning("Config tap device TUNSETOFFLOAD error");
			goto fail;
		}
		ndev->tap_ufo = false;
	}

	return 1;

fail:
	if ((ndev->tap_fd >= 0) || (!params->fd) )
		close(ndev->tap_fd);

	return 0;
}

static inline int tap_ops_tx(struct iovec *iov, u16 out, struct net_dev *ndev)
{
	return writev(ndev->tap_fd, iov, out);
}

static inline int tap_ops_rx(struct iovec *iov, u16 in, struct net_dev *ndev)
{
	return readv(ndev->tap_fd, iov, in);
}

static struct net_dev_operations tap_ops = {
	.rx	= tap_ops_rx,
	.tx	= tap_ops_tx,
};

static u8 *get_config(struct kvm *kvm, void *dev)
{
	struct net_dev *ndev = dev;

	return ((u8 *)(&ndev->config));
}

static u32 get_host_features(struct kvm *kvm, void *dev)
{
	u32 features;
	struct net_dev *ndev = dev;

	features = 1UL << VIRTIO_NET_F_MAC
		| 1UL << VIRTIO_NET_F_CSUM
		| 1UL << VIRTIO_NET_F_HOST_TSO4
		| 1UL << VIRTIO_NET_F_HOST_TSO6
		| 1UL << VIRTIO_NET_F_GUEST_TSO4
		| 1UL << VIRTIO_NET_F_GUEST_TSO6
		| 1UL << VIRTIO_RING_F_EVENT_IDX
		| 1UL << VIRTIO_RING_F_INDIRECT_DESC
		| 1UL << VIRTIO_NET_F_CTRL_VQ
		| 1UL << VIRTIO_NET_F_MRG_RXBUF
		| 1UL << (ndev->queue_pairs > 1 ? VIRTIO_NET_F_MQ : 0);

	/*
	 * The UFO feature for host and guest only can be enabled when the
	 * kernel has TAP UFO support.
	 */
	if (ndev->tap_ufo)
		features |= (1UL << VIRTIO_NET_F_HOST_UFO
				| 1UL << VIRTIO_NET_F_GUEST_UFO);

	return features;
}

static void set_guest_features(struct kvm *kvm, void *dev, u32 features)
{
	struct net_dev *ndev = dev;
	struct virtio_net_config *conf = &ndev->config;

	ndev->features = features;

	conf->status = virtio_host_to_guest_u16(&ndev->vdev, conf->status);
	conf->max_virtqueue_pairs = virtio_host_to_guest_u16(&ndev->vdev,
							     conf->max_virtqueue_pairs);
}

static void virtio_net_start(struct net_dev *ndev)
{
	if (ndev->mode == NET_MODE_TAP) {
		if (!virtio_net__tap_init(ndev))
			die_perror("TAP device initialized failed because");
	} else {
        die("Non-TAP mode unsupported");
	}
}

static void virtio_net_stop(struct net_dev *ndev)
{
	/* Undo whatever start() did */
	if (ndev->mode == NET_MODE_TAP)
		virtio_net__tap_exit(ndev);
	else
        die("Non-TAP mode unsupported");
}

static void notify_status(struct kvm *kvm, void *dev, u32 status)
{
	if (status & VIRTIO__STATUS_START)
		virtio_net_start(dev);
	else if (status & VIRTIO__STATUS_STOP)
		virtio_net_stop(dev);
}

static bool is_ctrl_vq(struct net_dev *ndev, u32 vq)
{
	return vq == (u32)(ndev->queue_pairs * 2);
}

static int init_vq(struct kvm *kvm, void *dev, u32 vq, u32 page_size, u32 align,
		   u32 pfn)
{
	struct net_dev_queue *net_queue;
	struct net_dev *ndev = dev;
	struct virt_queue *queue;
	void *p;

	net_queue	= &ndev->queues[vq];
	net_queue->id	= vq;
	net_queue->ndev	= ndev;
	queue		= &net_queue->vq;
	queue->pfn	= pfn;
	p		= virtio_get_vq(kvm, queue->pfn, page_size);

	vring_init(&queue->vring, VIRTIO_NET_QUEUE_SIZE, p, align);
	virtio_init_device_vq(&ndev->vdev, queue);

	mutex_init(&net_queue->lock);
	pthread_cond_init(&net_queue->cond, NULL);
	if (is_ctrl_vq(ndev, vq)) {
		pthread_create(&net_queue->thread, NULL, virtio_net_ctrl_thread,
			       net_queue);

		return 0;
	} else if (ndev->vhost_fd == 0 ) {
		if (vq & 1)
			pthread_create(&net_queue->thread, NULL,
				       virtio_net_tx_thread, net_queue);
		else
			pthread_create(&net_queue->thread, NULL,
				       virtio_net_rx_thread, net_queue);

		return 0;
    }
    
    return -1;
}

static void exit_vq(struct kvm *kvm, void *dev, u32 vq)
{
	struct net_dev *ndev = dev;
	struct net_dev_queue *queue = &ndev->queues[vq];

	/*
	 * Threads are waiting on cancellation points (readv or
	 * pthread_cond_wait) and should stop gracefully.
	 */
	pthread_cancel(queue->thread);
	pthread_join(queue->thread, NULL);
}

static int notify_vq(struct kvm *kvm, void *dev, u32 vq)
{
	struct net_dev *ndev = dev;

	virtio_net_handle_callback(kvm, ndev, vq);

	return 0;
}

static struct virt_queue *get_vq(struct kvm *kvm, void *dev, u32 vq)
{
	struct net_dev *ndev = dev;

	return &ndev->queues[vq].vq;
}

static int get_size_vq(struct kvm *kvm, void *dev, u32 vq)
{
	/* FIXME: dynamic */
	return VIRTIO_NET_QUEUE_SIZE;
}

static int set_size_vq(struct kvm *kvm, void *dev, u32 vq, int size)
{
	/* FIXME: dynamic */
	return size;
}

static int get_vq_count(struct kvm *kvm, void *dev)
{
	struct net_dev *ndev = dev;

	return ndev->queue_pairs * 2 + 1;
}

static struct virtio_ops net_dev_virtio_ops = {
	.get_config		= get_config,
	.get_host_features	= get_host_features,
	.set_guest_features	= set_guest_features,
	.get_vq_count		= get_vq_count,
	.init_vq		= init_vq,
	.exit_vq		= exit_vq,
	.get_vq			= get_vq,
	.get_size_vq		= get_size_vq,
	.set_size_vq		= set_size_vq,
	.notify_vq		= notify_vq,
	.notify_status		= notify_status,
};

static inline void str_to_mac(const char *str, char *mac)
{
	sscanf(str, "%hhx:%hhx:%hhx:%hhx:%hhx:%hhx",
		mac, mac+1, mac+2, mac+3, mac+4, mac+5);
}

static struct net_dev *lkvm_ndev = NULL;

static int virtio_net__init_one(struct virtio_net_params *params)
{
	int i, r;
	struct net_dev *ndev;
	struct virtio_ops *ops;
	enum virtio_trans trans = VIRTIO_DEFAULT_TRANS(params->kvm);

	ndev = calloc(1, sizeof(struct net_dev));
	if (ndev == NULL)
		return -ENOMEM;

	list_add_tail(&ndev->list, &ndevs);

	ops = malloc(sizeof(*ops));
	if (ops == NULL)
		return -ENOMEM;

	ndev->kvm = params->kvm;
	ndev->params = params;

	mutex_init(&ndev->mutex);
	ndev->queue_pairs = max(1, min(VIRTIO_NET_NUM_QUEUES, params->mq));
	ndev->config.status = VIRTIO_NET_S_LINK_UP;
	if (ndev->queue_pairs > 1)
		ndev->config.max_virtqueue_pairs = ndev->queue_pairs;

	for (i = 0 ; i < 6 ; i++) {
		ndev->config.mac[i]		= params->guest_mac[i];
		ndev->info.guest_mac.addr[i]	= params->guest_mac[i];
		ndev->info.host_mac.addr[i]	= params->host_mac[i];
	}

	ndev->mode = params->mode;
	if (ndev->mode == NET_MODE_TAP) {
		ndev->ops = &tap_ops;
		if (!virtio_net__tap_create(ndev))
			die_perror("You have requested a TAP device, but creation of one has failed because");
	} else {
        die("Non-TAP mode unsupported");
	}

	*ops = net_dev_virtio_ops;

	if (params->trans) {
		if (strcmp(params->trans, "mmio") == 0)
			trans = VIRTIO_MMIO;
		else
			pr_warning("virtio-net: Unknown transport method : %s, "
				   "falling back to %s.", params->trans,
				   virtio_trans_name(trans));
	}

    lkvm_ndev = ndev;
	r = virtio_init(params->kvm, ndev, &ndev->vdev, ops, trans,
			PCI_DEVICE_ID_VIRTIO_NET, VIRTIO_ID_NET, PCI_CLASS_NET);
	if (r < 0) {
		free(ops);
		return r;
	}

	return 0;
}

int virtio_net__init(struct kvm *kvm)
{
	int r;

    static struct virtio_net_params net_params;

    net_params = (struct virtio_net_params) {
        .kvm		= kvm,
            .script		= "none",
            .mode		= NET_MODE_TAP,
            .tapif		= "vmtap0",
    };
    str_to_mac(kvm->cfg.guest_mac, net_params.guest_mac);
    str_to_mac(kvm->cfg.host_mac, net_params.host_mac);

    r = virtio_net__init_one(&net_params);
    if (r < 0)
        goto cleanup;

    return 0;

cleanup:
    return r;
}

static struct kvm fake_kvm = {
	.cfg = {
        .guest_mac = "52:54:00:12:34:88",
        .host_mac = "52:54:00:12:34:99",
    },
};

void lkvm_net_init(void);
void lkvm_net_init(void) {
    virtio_net__init(&fake_kvm);
}

extern void virtio_mmio_read(void *ndev, u64 offset, u8 *data, u32 len);
extern void virtio_mmio_write(void *ndev, u64 offset, u8 *data, u32 len);

void lkvm_net_mmio_read(u64 offset, u8 *data, u32 len);
void lkvm_net_mmio_read(u64 offset, u8 *data, u32 len)
{
    virtio_mmio_read(&lkvm_ndev->vdev, offset, data, len);
}

void lkvm_net_mmio_write(u64 offset, u8 *data, u32 len);
void lkvm_net_mmio_write(u64 offset, u8 *data, u32 len)
{
    virtio_mmio_write(&lkvm_ndev->vdev, offset, data, len);
}
