proc main_test { } {
    # Test the binary
    expect "ubuntu@ubuntu:~/laputa\\\$"

    send "./laputa --smp 4\n"
    expect {
            "Error: please provide memory size by using --memory or config files." {
        }
        timeout {
            -1
        }
    }

    send "./laputa --memory 128\n"
    expect {
            "Error: please provide vcpu count by using --smp or config files." {
        }
        timeout {
            -1
        }
    }

    send "./laputa --smp 4 --memory 128\n"
    expect {
            "Error: please provide kernel image by using --kernel or config files." {
        }
        timeout {
            -1
        }
    }
}
