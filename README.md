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

## Build linux
```
./scripts/local/build_linux.sh
```

## Build opensbi
```
./scripts/local/build_opensbi.sh
```

## Build laputa
```
./scripts/local/docker_build_laputa.sh
```

## Test laputa
```
./scripts/local/docker_test_laputa.sh
```
Or if you want to run test in verbose mode, run:
```
./scripts/local/docker_test_laputa.sh --nocapture
```
## Build testing app
```
./scripts/local/docker_build_app.sh
```

## Test testing app
```
./scripts/local/docker_test_app.sh
```
## Update Docker
When building laputa, rust package in docker will always be reinstalled, which is quite time-consuming.
The scripts below will help update rust package in docker image.
```
./scripts/local/docker_update.sh
``` 
## Debug laputa
The following script will boot qemu with -s -S arguments
```
./scripts/local/docker_debug_laputa.sh
```
If you want to debug certain test case, for example, vm::virtualmachine::tests::test_vtimer_sret, you can run
```
./scripts/local/docker_debug_laputa.sh vm::virtualmachine::tests::test_vtimer_sret
```

## Example
You can try linux VM with following commands:
```
./scripts/local/boot.sh

./chroot.sh

chmod +x ./laputa_linux.sh
./laputa_linux.sh
```