proc main_test_multi_vm_2_2 { } {
    # Test the binary
    expect "root@(none)"

    send "cd .. \n"
    send "./multi-vm-test/multi-vm-2-vcpu-ip7.sh < /dev/null & \n\n"
    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    send "./multi-vm-test/multi-vm-2-vcpu-ip8.sh < /dev/null & \n\n"
    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }
}

proc main_test_multi_vm_2_4 { } {
    # Test the binary
    expect "root@(none)"

    send "cd .. \n"
    send "./multi-vm-test/multi-vm-2-vcpu-ip7.sh < /dev/null & \n\n"
    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    send "./multi-vm-test/multi-vm-2-vcpu-ip8.sh < /dev/null & \n\n"
    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    send "./multi-vm-test/multi-vm-2-vcpu-ip9.sh < /dev/null & \n\n"
    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    send "./multi-vm-test/multi-vm-2-vcpu-ip10.sh < /dev/null & \n\n"
    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }
}

proc main_test_multi_vm_2_8 { } {
    # Test the binary
    expect "root@(none)"

    send "cd .. \n"
    send "./multi-vm-test/multi-vm-2-vcpu-ip7.sh < /dev/null & \n\n"
    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    send "./multi-vm-test/multi-vm-2-vcpu-ip8.sh < /dev/null & \n\n"
    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    send "./multi-vm-test/multi-vm-2-vcpu-ip9.sh < /dev/null & \n\n"
    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    send "./multi-vm-test/multi-vm-2-vcpu-ip10.sh < /dev/null & \n\n"
    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    send "./multi-vm-test/multi-vm-2-vcpu-ip11.sh < /dev/null & \n\n"
    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    send "./multi-vm-test/multi-vm-2-vcpu-ip12.sh < /dev/null & \n\n"
    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    send "./multi-vm-test/multi-vm-2-vcpu-ip13.sh < /dev/null & \n\n"
    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    send "./multi-vm-test/multi-vm-2-vcpu-ip14.sh < /dev/null & \n\n"
    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }
}

proc main_test_multi_vm_host_2_8 { } {
    # Wait for VMs to start up
    sleep 100

    # Test VM-0
    set x 0

    while {$x < 1} {
        set timeout 1000

        send "ssh -v root@192.168.254.7 -o StrictHostKeyChecking=no \n"
        expect {
            "s password:" {
                incr x;
                break;
            }

            "Connection closed by" {
                continue;
            }

            "Connection refused" {
                continue;
            }

            "No route to host" {
                exit -1
            }

            timeout {
                exit -1
            }
        }
    }

    send "123\n"

    expect {
        "#" {
        }

        timeout {
            exit -1
        }
    }

    send "ip a \n"
    expect {
        "192.168.254.7" {
        }

        timeout {
            exit -1
        }
    }

    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    set timeout 500

    send "hackbench \n" 
    expect {
        "Time:" {
        }
        
        timeout {
            exit -1
        }
    }

    send "exit \n"
    expect {
        "Connection to 192.168.254.7 closed." {
        }

        timeout {
            exit -1
        }
    }

    send " \n"

    sleep 2

    # Test VM-1
    set x 0

    while {$x < 1} {
        set timeout 1000

        send "ssh -v root@192.168.254.8 -o StrictHostKeyChecking=no \n"
        expect {
            "s password:" {
                incr x;
                break;
            }

            "Connection closed by" {
                continue;
            }

            "Connection refused" {
                continue;
            }

            "No route to host" {
                exit -1
            }

            timeout {
                exit -1
            }
        }
    }

    send "123\n"

    expect {
        "#" {
        }

        timeout {
            exit -1
        }
    }

    send "ip a \n"
    expect {
        "192.168.254.8" {
        }

        timeout {
            exit -1
        }
    }

    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    set timeout 1000

    send "cd .. \n"
    expect {
        "#" {
        }
        
        timeout {
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

    send "exit \n"
    expect {
        "Connection to 192.168.254.8 closed." {
        }

        timeout {
            exit -1
        }
    }

    send " \n"

    sleep 2

    # Test VM-2
    set x 0

    while {$x < 1} {
        set timeout 1000

        send "ssh -v root@192.168.254.9 -o StrictHostKeyChecking=no \n"
        expect {
            "s password:" {
                incr x;
                break;
            }

            "Connection closed by" {
                continue;
            }

            "Connection refused" {
                continue;
            }

            "No route to host" {
                exit -1
            }

            timeout {
                exit -1
            }
        }
    }

    send "123\n"

    expect {
        "#" {
        }

        timeout {
            exit -1
        }
    }

    send "ip a \n"
    expect {
        "192.168.254.9" {
        }

        timeout {
            exit -1
        }
    }

    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    set timeout 500

    send "hackbench \n" 
    expect {
        "Time:" {
        }
        
        timeout {
            exit -1
        }
    }

    send "exit \n"
    expect {
        "Connection to 192.168.254.9 closed." {
        }

        timeout {
            exit -1
        }
    }

    send " \n"

    sleep 2

    # Test VM-3
    set x 0

    while {$x < 1} {
        set timeout 1000

        send "ssh -v root@192.168.254.10 -o StrictHostKeyChecking=no \n"
        expect {
            "s password:" {
                incr x;
                break;
            }

            "Connection closed by" {
                continue;
            }

            "Connection refused" {
                continue;
            }

            "No route to host" {
                exit -1
            }

            timeout {
                exit -1
            }
        }
    }

    send "123\n"

    expect {
        "#" {
        }

        timeout {
            exit -1
        }
    }

    send "ip a \n"
    expect {
        "192.168.254.10" {
        }

        timeout {
            exit -1
        }
    }

    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    set timeout 1000

    send "cd .. \n"
    expect {
        "#" {
        }
        
        timeout {
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

    send "exit \n"
    expect {
        "Connection to 192.168.254.10 closed." {
        }

        timeout {
            exit -1
        }
    }

    send " \n"

    sleep 2

    # Test VM-4
    set x 0

    while {$x < 1} {
        set timeout 1000

        send "ssh -v root@192.168.254.11 -o StrictHostKeyChecking=no \n"
        expect {
            "s password:" {
                incr x;
                break;
            }

            "Connection closed by" {
                continue;
            }

            "Connection refused" {
                continue;
            }

            "No route to host" {
                exit -1
            }

            timeout {
                exit -1
            }
        }
    }

    send "123\n"

    expect {
        "#" {
        }

        timeout {
            exit -1
        }
    }

    send "ip a \n"
    expect {
        "192.168.254.11" {
        }

        timeout {
            exit -1
        }
    }

    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    set timeout 500

    send "hackbench \n" 
    expect {
        "Time:" {
        }
        
        timeout {
            exit -1
        }
    }

    send "exit \n"
    expect {
        "Connection to 192.168.254.11 closed." {
        }

        timeout {
            exit -1
        }
    }

    send " \n"

    sleep 2

    # Test VM-5
    set x 0

    while {$x < 1} {
        set timeout 1000

        send "ssh -v root@192.168.254.12 -o StrictHostKeyChecking=no \n"
        expect {
            "s password:" {
                incr x;
                break;
            }

            "Connection closed by" {
                continue;
            }

            "Connection refused" {
                continue;
            }

            "No route to host" {
                exit -1
            }

            timeout {
                exit -1
            }
        }
    }

    send "123\n"

    expect {
        "#" {
        }

        timeout {
            exit -1
        }
    }

    send "ip a \n"
    expect {
        "192.168.254.12" {
        }

        timeout {
            exit -1
        }
    }

    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    set timeout 1000

    send "cd .. \n"
    expect {
        "#" {
        }
        
        timeout {
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

    send "exit \n"
    expect {
        "Connection to 192.168.254.12 closed." {
        }

        timeout {
            exit -1
        }
    }

    send " \n"

    sleep 2

    # Test VM-6
    set x 0

    while {$x < 1} {
        set timeout 1000

        send "ssh -v root@192.168.254.13 -o StrictHostKeyChecking=no \n"
        expect {
            "s password:" {
                incr x;
                break;
            }

            "Connection closed by" {
                continue;
            }

            "Connection refused" {
                continue;
            }

            "No route to host" {
                exit -1
            }

            timeout {
                exit -1
            }
        }
    }

    send "123\n"

    expect {
        "#" {
        }

        timeout {
            exit -1
        }
    }

    send "ip a \n"
    expect {
        "192.168.254.13" {
        }

        timeout {
            exit -1
        }
    }

    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    set timeout 1000

    send "hackbench \n" 
    expect {
        "Time:" {
        }
        
        timeout {
            exit -1
        }
    }

    send "exit \n"
    expect {
        "Connection to 192.168.254.13 closed." {
        }

        timeout {
            exit -1
        }
    }

    send " \n"

    sleep 2

    # Test VM-7
    set x 0

    while {$x < 1} {
        set timeout 1000

        send "ssh -v root@192.168.254.14 -o StrictHostKeyChecking=no \n"
        expect {
            "s password:" {
                incr x;
                break;
            }

            "Connection closed by" {
                continue;
            }

            "Connection refused" {
                continue;
            }

            "No route to host" {
                exit -1
            }

            timeout {
                exit -1
            }
        }
    }

    send "123\n"

    expect {
        "#" {
        }

        timeout {
            exit -1
        }
    }

    send "ip a \n"
    expect {
        "192.168.254.14" {
        }

        timeout {
            exit -1
        }
    }

    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    set timeout 1000

    send "cd .. \n"
    expect {
        "#" {
        }
        
        timeout {
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

    send "exit \n"
    expect {
        "Connection to 192.168.254.14 closed." {
        }

        timeout {
            exit -1
        }
    }
}

proc main_test_multi_vm_host_2_2 { } {
    # Wait for VMs to start up
    sleep 100

    # Test VM-0
    send "ssh -v root@192.168.254.7 -o StrictHostKeyChecking=no \n"
    expect {
        "password" {
            send "123\n"

            expect {
                "#" {
                    send "ip a \n"

                    expect {
                        "192.168.254.7" {
                            send "exit \n"

                            expect {
                                "Connection to 192.168.254.7 closed." {}
                                "Connection closed by" {}

                                timeout {
                                    exit -1
                                }
                            }
                        }

                        "Connection to 192.168.254.7 closed." {}
                        "Connection closed by" {}

                        timeout {
                            exit -1
                        }
                    }
                }

                "Connection to 192.168.254.7 closed." {}
                "Connection closed by" {}

                timeout {
                    exit -1
                }
            }
        }

        "No route to host" {}
        "Connection closed by" {}
        "Connection to 192.168.254.7 closed." {}
        
        timeout {
            exit -1
        }
    }

    send " \n"

    sleep 2

    # Test VM-1
    send "ssh -v root@192.168.254.8 -o StrictHostKeyChecking=no \n"
    expect {
        "password" {
        }

        timeout {
            exit -1
        }
    }

    send "123\n"

    expect {
        "#" {
        }

        timeout {
            exit -1
        }
    }

    send "ip a \n"
    expect {
        "192.168.254.8" {
        }

        timeout {
            exit -1
        }
    }

    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    set timeout 1000

    send "hackbench \n" 
    expect {
        "Time:" {
        }
        
        timeout {
            exit -1
        }

    }

    send "cd .. \n"
    expect {
        "#" {
        }
        
        timeout {
            exit -1
        }
    }

    send "./lmbench.sh \n"
    expect {
        "Simple syscall" {
        }

        timeout {
            exit -1
        }
    }

    send "exit \n"
    expect {
        "Connection to 192.168.254.8 closed." {
        }

        timeout {
            exit -1
        }
    }
}

proc main_test_multi_vm_host_2_4 { } {
    # Wait for VMs to start up
    sleep 100

    # Test VM-0
    send "ssh -v root@192.168.254.7 -o StrictHostKeyChecking=no \n"
    expect {
        "password" {
            send "123\n"

            expect {
                "#" {
                    send "ip a \n"

                    expect {
                        "192.168.254.7" {
                            send "exit \n"

                            expect {
                                "Connection to 192.168.254.7 closed." {}
                                "Connection closed by" {}

                                timeout {
                                    exit -1
                                }
                            }
                        }

                        "Connection to 192.168.254.7 closed." {}
                        "Connection closed by" {}

                        timeout {
                            exit -1
                        }
                    }
                }

                "Connection to 192.168.254.7 closed." {}
                "Connection closed by" {}

                timeout {
                    exit -1
                }
            }
        }

        "No route to host" {}
        "Connection closed by" {}
        "Connection to 192.168.254.7 closed." {}

        timeout {
            exit -1
        }
    }

    send " \n"

    sleep 2

    # Test VM-1
    send "ssh -v root@192.168.254.8 -o StrictHostKeyChecking=no \n"
    expect {
        "password" {
            send "123\n"

            expect {
                "#" {
                    send "ip a \n"

                    expect {
                        "192.168.254.8" {
                            send "exit \n"

                            expect {
                                "Connection to 192.168.254.8 closed." {}
                                "Connection closed by" {}

                                timeout {
                                    exit -1
                                }
                            }
                        }

                        "Connection to 192.168.254.8 closed." {}
                        "Connection closed by" {}

                        timeout {
                            exit -1
                        }
                    }
                }

                "Connection to 192.168.254.8 closed." {}
                "Connection closed by" {}

                timeout {
                    exit -1
                }
            }
        }

        "No route to host" {}
        "Connection closed by" {}
        "Connection to 192.168.254.8 closed." {}
        
        timeout {
            exit -1
        }
    }

    send " \n"

    sleep 2

    # Test VM-2
    send "ssh -v root@192.168.254.9 -o StrictHostKeyChecking=no \n"
    expect {
        "password" {
            send "123\n"

            expect {
                "#" {
                    send "ip a \n"

                    expect {
                        "192.168.254.9" {
                            send "exit \n"

                            expect {
                                "Connection to 192.168.254.9 closed." {}
                                "Connection closed by" {}

                                timeout {
                                    exit -1
                                }
                            }
                        }

                        "Connection to 192.168.254.9 closed." {}
                        "Connection closed by" {}

                        timeout {
                            exit -1
                        }
                    }
                }

                "Connection to 192.168.254.9 closed." {}
                "Connection closed by" {}

                timeout {
                    exit -1
                }
            }
        }

        "No route to host" {}
        "Connection closed by" {}
        "Connection to 192.168.254.9 closed." {}
        
        timeout {
            exit -1
        }
    }

    send " \n"

    sleep 2

    # Test VM-3
    send "ssh -v root@192.168.254.10 -o StrictHostKeyChecking=no \n"
    expect {
        "password" {
        }

        timeout {
            exit -1
        }
    }

    send "123\n"

    expect {
        "#" {
        }

        timeout {
            exit -1
        }
    }

    send "ip a \n"
    expect {
        "192.168.254.10" {
        }

        timeout {
            exit -1
        }
    }

    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    set timeout 1000

    send "hackbench \n" 
    expect {
        "Time:" {
        }
        
        timeout {
            exit -1
        }

    }

    send "cd .. \n"
    expect {
        "#" {
        }
        
        timeout {
            exit -1
        }
    }

    send "./lmbench.sh \n"
    expect {
        "Simple syscall" {
        }

        timeout {
            exit -1
        }
    }

    send "exit \n"
    expect {
        "Connection to 192.168.254.10 closed." {
        }

        timeout {
            exit -1
        }
    }
}

proc main_test_multi_vm_host_ip_7_start { } {
    # Connect VM-0 by ssh
    set x 0

    while {$x < 1} {
        set timeout 1000

        send "ssh -v root@192.168.254.7 -o StrictHostKeyChecking=no \n"
        expect {
            "s password:" {
                incr x;
                break;
            }

            "Connection closed by" {
                continue;
            }

            "port 22: Connection timed out" {
                continue;
            }

            "Connection refused" {
                continue;
            }

            "No route to host" {
                exit -1
            }

            timeout {
                exit -1
            }
        }
    }

    send "123\n"

    expect {
        "#" {
        }

        timeout {
            exit -1
        }
    }

    send "ip a \n"
    expect {
        "192.168.254.7" {
        }

        timeout {
            exit -1
        }
    }

    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    send "hackbench \n"
}

proc main_test_multi_vm_host_ip_8_start { } {
    # Connect VM-1 by ssh
    set x 0

    while {$x < 1} {
        set timeout 1000

        send "ssh -v root@192.168.254.8 -o StrictHostKeyChecking=no \n"
        expect {
            "s password:" {
                incr x;
                break;
            }

            "Connection closed by" {
                continue;
            }

            "port 22: Connection timed out" {
                continue;
            }

            "Connection refused" {
                continue;
            }

            "No route to host" {
                exit -1
            }

            timeout {
                exit -1
            }
        }
    }

    send "123\n"

    expect {
        "#" {
        }

        timeout {
            exit -1
        }
    }

    send "ip a \n"
    expect {
        "192.168.254.8" {
        }

        timeout {
            exit -1
        }
    }

    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    send "hackbench \n"
}

proc main_test_multi_vm_host_ip_9_start { } {
    # Connect VM-2 by ssh
    set x 0

    while {$x < 1} {
        set timeout 1000

        send "ssh -v root@192.168.254.9 -o StrictHostKeyChecking=no \n"
        expect {
            "s password:" {
                incr x;
                break;
            }

            "Connection closed by" {
                continue;
            }

            "Connection refused" {
                continue;
            }

            "port 22: Connection timed out" {
                continue;
            }

            "No route to host" {
                exit -1
            }

            timeout {
                exit -1
            }
        }
    }

    send "123\n"

    expect {
        "#" {
        }

        timeout {
            exit -1
        }
    }

    send "ip a \n"
    expect {
        "192.168.254.9" {
        }

        timeout {
            exit -1
        }
    }

    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    send "hackbench \n"
}

proc main_test_multi_vm_host_ip_10_start { } {
    # Connect VM-3 by ssh
    set x 0

    while {$x < 1} {
        set timeout 1000

        send "ssh -v root@192.168.254.10 -o StrictHostKeyChecking=no \n"
        expect {
            "s password:" {
                incr x;
                break;
            }

            "Connection closed by" {
                continue;
            }

            "port 22: Connection timed out" {
                continue;
            }

            "Connection refused" {
                continue;
            }

            "No route to host" {
                exit -1
            }

            timeout {
                exit -1
            }
        }
    }

    send "123\n"

    expect {
        "#" {
        }

        timeout {
            exit -1
        }
    }

    send "ip a \n"
    expect {
        "192.168.254.10" {
        }

        timeout {
            exit -1
        }
    }

    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    send "hackbench \n"
}

# lmbench VMs
proc main_test_multi_vm_host_ip_11_start { } {
    # Connect VM-4 by ssh
    set x 0

    while {$x < 1} {
        set timeout 1000

        send "ssh -v root@192.168.254.11 -o StrictHostKeyChecking=no \n"
        expect {
            "s password:" {
                incr x;
                break;
            }

            "Connection closed by" {
                continue;
            }

            "port 22: Connection timed out" {
                continue;
            }

            "Connection refused" {
                continue;
            }

            "No route to host" {
                exit -1
            }

            timeout {
                exit -1
            }
        }
    }

    send "123\n"

    expect {
        "#" {
        }

        timeout {
            exit -1
        }
    }

    send "ip a \n"
    expect {
        "192.168.254.11" {
        }

        timeout {
            exit -1
        }
    }

    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    send "cd .. \n"
    expect {
        "#" {
        }
        
        timeout {
            exit -1
        }
    }

    send "./lmbench.sh \n"
}

proc main_test_multi_vm_host_ip_12_start { } {
    # Connect VM-5 by ssh
    set x 0

    while {$x < 1} {
        set timeout 1000

        send "ssh -v root@192.168.254.12 -o StrictHostKeyChecking=no \n"
        expect {
            "s password:" {
                incr x;
                break;
            }

            "Connection closed by" {
                continue;
            }

            "port 22: Connection timed out" {
                continue;
            }

            "Connection refused" {
                continue;
            }

            "No route to host" {
                exit -1
            }

            timeout {
                exit -1
            }
        }
    }

    send "123\n"

    expect {
        "#" {
        }

        timeout {
            exit -1
        }
    }

    send "ip a \n"
    expect {
        "192.168.254.12" {
        }

        timeout {
            exit -1
        }
    }

    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    send "cd .. \n"
    expect {
        "#" {
        }
        
        timeout {
            exit -1
        }
    }

    send "./lmbench.sh \n"
}

proc main_test_multi_vm_host_ip_13_start { } {
    # Connect VM-6 by ssh
    set x 0

    while {$x < 1} {
        set timeout 1000

        send "ssh -v root@192.168.254.13 -o StrictHostKeyChecking=no \n"
        expect {
            "s password:" {
                incr x;
                break;
            }

            "Connection closed by" {
                continue;
            }

            "port 22: Connection timed out" {
                continue;
            }

            "Connection refused" {
                continue;
            }

            "No route to host" {
                exit -1
            }

            timeout {
                exit -1
            }
        }
    }

    send "123\n"

    expect {
        "#" {
        }

        timeout {
            exit -1
        }
    }

    send "ip a \n"
    expect {
        "192.168.254.13" {
        }

        timeout {
            exit -1
        }
    }

    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    send "cd .. \n"
    expect {
        "#" {
        }
        
        timeout {
            exit -1
        }
    }

    send "./lmbench.sh \n"
}

proc main_test_multi_vm_host_ip_14_start { } {
    # Connect VM-7 by ssh
    set x 0

    while {$x < 1} {
        set timeout 1000

        send "ssh -v root@192.168.254.14 -o StrictHostKeyChecking=no \n"
        expect {
            "s password:" {
                incr x;
                break;
            }

            "Connection closed by" {
                continue;
            }

            "Connection refused" {
                continue;
            }

            "port 22: Connection timed out" {
                continue;
            }

            "No route to host" {
                exit -1
            }

            timeout {
                exit -1
            }
        }
    }

    send "123\n"

    expect {
        "#" {
        }

        timeout {
            exit -1
        }
    }

    send "ip a \n"
    expect {
        "192.168.254.14" {
        }

        timeout {
            exit -1
        }
    }

    expect {
        "root@(none)" {
        }

        timeout {
            exit -1
        }
    }

    send "cd .. \n"
    expect {
        "#" {
        }
        
        timeout {
            exit -1
        }
    }

    send "./lmbench.sh \n"
}

proc main_test_multi_vm_host_ip_7_check { } {
    # Check the result of workload of VM-0
    expect {
        "Time:" {
        }
        
        timeout {
            exit -1
        }
    }

    send "exit \n"
    expect {
        "Connection to 192.168.254.7 closed." {
        }

        timeout {
            exit -1
        }
    }

    send " \n"

    sleep 2
}

proc main_test_multi_vm_host_ip_8_check { } {
    # Check the result of workload of VM-1
    expect {
        "Time:" {
        }
        
        timeout {
            exit -1
        }
    }

    send "exit \n"
    expect {
        "Connection to 192.168.254.8 closed." {
        }

        timeout {
            exit -1
        }
    }

    send " \n"

    sleep 2
}

proc main_test_multi_vm_host_ip_9_check { } {
    # Check the result of workload of VM-2
    expect {
        "Time:" {
        }
        
        timeout {
            exit -1
        }
    }

    send "exit \n"
    expect {
        "Connection to 192.168.254.9 closed." {
        }

        timeout {
            exit -1
        }
    }

    send " \n"

    sleep 2
}

proc main_test_multi_vm_host_ip_10_check { } {
    # Check the result of workload of VM-3
    expect {
        "Time:" {
        }
        
        timeout {
            exit -1
        }
    }

    send "exit \n"
    expect {
        "Connection to 192.168.254.10 closed." {
        }

        timeout {
            exit -1
        }
    }

    send " \n"

    sleep 2
}

proc main_test_multi_vm_host_ip_11_check { } {
    # Check the result of workload of VM-4
    expect {
        "Simple syscall" {
        }

        timeout {
            puts "Timeout by lmbench"
            exit -1
        }
    }

    send "exit \n"
    expect {
        "Connection to 192.168.254.11 closed." {
        }

        timeout {
            exit -1
        }
    }

    send " \n"

    sleep 2
}

proc main_test_multi_vm_host_ip_12_check { } {
    # Check the result of workload of VM-5
    expect {
        "Simple syscall" {
        }

        timeout {
            puts "Timeout by lmbench"
            exit -1
        }
    }

    send "exit \n"
    expect {
        "Connection to 192.168.254.12 closed." {
        }

        timeout {
            exit -1
        }
    }

    send " \n"

    sleep 2
}

proc main_test_multi_vm_host_ip_13_check { } {
    # Check the result of workload of VM-6
    expect {
        "Simple syscall" {
        }

        timeout {
            puts "Timeout by lmbench"
            exit -1
        }
    }

    send "exit \n"
    expect {
        "Connection to 192.168.254.13 closed." {
        }

        timeout {
            exit -1
        }
    }

    send " \n"

    sleep 2
}

proc main_test_multi_vm_host_ip_14_check { } {
    # Check the result of workload of VM-7
    expect {
        "Simple syscall" {
        }

        timeout {
            puts "Timeout by lmbench"
            exit -1
        }
    }

    send "exit \n"
    expect {
        "Connection to 192.168.254.14 closed." {
        }

        timeout {
            exit -1
        }
    }

    send " \n"

    sleep 2
}