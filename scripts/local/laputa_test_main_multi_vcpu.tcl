proc main_test_multi_vcpu { } {
    # Test the binary
    expect ":/laputa#"

    send "./laputa --smp 2 --initrd ./test-files-laputa/rootfs-net.img --dtb ./test-files-laputa/smp2-io.dtb  --kernel ./test-files-laputa/Image --memory 1024 --machine laputa_virt\n"
    expect {
        "Busybox Rootfs" {
            send "\n ls \n"
            expect {
                "guest-net.sh" {}
            }
        }
        
        timeout {
            exit -1
        }vivado
    }

    send "/guest-net.sh \n ip a \n"
    expect {
        "eth0: <BROADCAST,MULTICAST,UP,LOWER_UP>" {
        }

        timeout {
            exit -1
        }
    }

    send "poweroff -f \n"
    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }
}
