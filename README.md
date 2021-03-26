# LAPUTA 

## Prepare
All the passwords is ***ipads123***
```
sudo apt install qemu-system-misc opensbi u-boot-qemu qemu-utils 
./scripts/local/prepare.sh
```
## Boot
```
./scripts/local/boot.sh
``` 

All the following commands depends on the vm boot. 

## Build
```
./scripts/local/docker_build.sh
```

## Test
```
./scripts/local/docker_test.sh
```
