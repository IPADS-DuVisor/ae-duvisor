sudo docker run -it \
    -v $(pwd):/home/ubuntu/laputa \
    -w /home/ubuntu/laputa \
    --network host \
    --privileged=true 1197744123/laputa:v4 \
    ./scripts/local/copy_laputa_to_vm.sh

docker commit `docker container ls -a --latest -q` 1197744123/laputa:v4
