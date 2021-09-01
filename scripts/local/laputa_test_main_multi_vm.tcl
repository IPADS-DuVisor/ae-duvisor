proc main_test_multi_vm { } {
    # Test the binary
    expect ":/laputa#"

    send "cd .. \n"
    send "./multi-vm-1.sh < /dev/null & \n"
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

    send "./multi-vm-1-8.sh < /dev/null & \n"
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

proc main_test_multi_vm_host { } {
    # Test the binary

    send "ssh root@192.168.254.7 \n"
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

    send "ssh root@192.168.254.8 \n"
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

    send "exit \n"
    expect {
        "Connection to 192.168.254.8 closed." {
        }

        timeout {
            exit -1
        }
    }
}
