proc main_test_multi_vcpu_8 { } {
    expect {
        "root@(none):/#" {
        }

        timeout {
            exit -1
        }
    }

    send "cd laputa && ./laputa --smp 8 --initrd ./test-files-laputa/rootfs-net.img --dtb ./test-files-laputa/smp8-io.dtb  --kernel ./test-files-laputa/Image --memory 128 --machine laputa_virt\n"
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

    set timeout 500
    send "mount /dev/vda /root && chroot root \n"
    expect "#"

    send "hackbench \n" 
    expect {
        "Time:" {
        }
        
        timeout {
            puts "Timeout by hackbench"
            exit -1
        }
    }

    send "./lmbench.sh \n"
    expect {
        "Simple syscall" {
        }

        timeout {
            puts "Timeout by lmbench"
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
