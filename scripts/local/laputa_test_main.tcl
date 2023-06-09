proc main_test { } {
    # Test the binary
    expect ":/laputa#"

    send "./laputa --smp 4\n"
    expect {
            "please set memory size by using --memory or config files." {
        }
        timeout {
            exit -1
        }
    }

    send "./laputa --memory 128\n"
    expect {
            "please set vcpu count by using --smp or config files." {
        }
        timeout {
            exit -1
        }
    }

    send "./laputa --smp 4 --memory 128\n"
    expect {
            "please set kernel image by using --kernel or config files." {
        }
        timeout {
            exit -1
        }
    }

    set timeout 300

    send "./laputa --smp 1 --initrd ./test-files-laputa/rootfs-net.img --dtb ./test-files-laputa/vmlinux.dtb  --kernel ./test-files-laputa/Image --memory 1024 --machine laputa_virt\n"
    expect {
        "Busybox Rootfs" {
            send "\n ls \n"
            expect {
                "guest-net.sh" {}
            }
        }
        
        timeout {
            exit -1
        }
    }

    send "/guest-net.sh \n ip a \n"
    expect {
        "eth0: <BROADCAST,MULTICAST,UP,LOWER_UP>" {
        }

        timeout {
            exit -1
        }
    }

    send "sync \n"
    expect {
        "#" {
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
