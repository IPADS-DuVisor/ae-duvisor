HOST=${HOST:-"127.0.0.1"}
TIMES=${TIMES:-3}

n=0

while [ $n -lt $TIMES ]; do 
    ./iperf3 -c $HOST
n=$(( n + 1 ))
done
