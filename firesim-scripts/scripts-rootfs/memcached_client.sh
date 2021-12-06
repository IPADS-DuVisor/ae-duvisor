#cd /home/ubuntu/memtier_benchmark
HOST=${HOST:-127.0.0.1}
TIMES=${TIMES:-3}
#./memtier_benchmark -s $HOST -p 11211 -P memcache_binary
cd /home/ubuntu/libmemcached-1.0.18/clients
n=0

while [ $n -lt $TIMES ]; do 
    ./memaslap -s $HOST:11211 -v 0.2 -e 0.05 -B -T 4 -c 128 -x 100000
    n=$(( n + 1 ))
done
