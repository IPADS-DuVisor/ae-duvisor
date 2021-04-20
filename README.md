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
## Build laputa
```
./scripts/local/docker_build_laputa.sh
```

## Test laputa
```
./scripts/local/docker_test_laputa.sh
```
