# LAPUTA 

## Prepare
All the passwords is ***ipads123***
```
# There are some update, please remove prepare directory
rm -r prepare

./scripts/local/prepare.sh
```

## Build qemu
```
./scripts/local/build_qemu.sh
```

## Build linux (Optional, run if you make changes to linux-laputa)
```
./scripts/local/build_linux.sh
```
## Boot
Username: ubuntu

Password: ipads123
```
./scripts/local/docker_boot.sh
``` 

All the following commands depends on the vm boot. 

## Install linux (Optional, run if you make changes to linux-laputa)
``` 
./scripts/local/docker_install_linux.sh
```

This will take long, wait until the vm reboot.

## Build laputa
```
./scripts/local/docker_build_laputa.sh
```

## Test laputa
```
./scripts/local/docker_test_laputa.sh
```
