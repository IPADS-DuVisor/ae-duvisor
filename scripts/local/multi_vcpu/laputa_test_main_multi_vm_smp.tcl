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
    send "ssh root@192.168.254.7 -o StrictHostKeyChecking=no \n"
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
        "Connection to 192.168.254.7 closed." {
        }

        timeout {
            exit -1
        }
    }

    send " \n"

    sleep 2

    # Test VM-1
    send "ssh root@192.168.254.8 -o StrictHostKeyChecking=no \n"
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

    send " \n"

    sleep 2

    # Test VM-2
    send "ssh root@192.168.254.9 -o StrictHostKeyChecking=no \n"
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
        "Connection to 192.168.254.9 closed." {
        }

        timeout {
            exit -1
        }
    }

    send " \n"

    sleep 2

    # Test VM-3
    send "ssh root@192.168.254.10 -o StrictHostKeyChecking=no \n"
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

    send " \n"

    sleep 2

    # Test VM-4
    send "ssh root@192.168.254.11 -o StrictHostKeyChecking=no \n"
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
        "Connection to 192.168.254.11 closed." {
        }

        timeout {
            exit -1
        }
    }

    send " \n"

    sleep 2

    # Test VM-5
    send "ssh root@192.168.254.12 -o StrictHostKeyChecking=no \n"
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
    send "ssh root@192.168.254.13 -o StrictHostKeyChecking=no \n"
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
    send "ssh root@192.168.254.14 -o StrictHostKeyChecking=no \n"
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