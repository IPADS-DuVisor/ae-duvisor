proc main_test { } {
    # update to test more functions
    expect "ubuntu@ubuntu:~/laputa\\\$"

    send "./laputa\n"
    expect {
        "Hello from laputa" {
            
        }
        timeout {
            -1
        }
    }
}
