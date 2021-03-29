# LAPUTA 

## Prepare
All the passwords is ***ipads123***
```
./scripts/local/prepare.sh
```

## Build qemu and linux kernel
```
./scripts/local/docker_build_qemu.sh
./scripts/local/docker_build_linux.sh
```

***TODO: Right now， it just builds linux and does nothing else. Later on, it should copy the built linux kernel image into ubuntu.***

## Boot
Username: ubuntu

Password: ipads123
```
./scripts/local/docker_boot.sh
``` 

All the following commands depends on the vm boot. 

## Build laputa
```
./scripts/local/docker_build_laputa.sh
```

## Test laputa
```
./scripts/local/docker_test.sh
```
