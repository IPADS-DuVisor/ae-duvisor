TIMES=${TIMES:-3}

n=0

while [ $n -lt $TIMES ]; do 
    /home/ubuntu/sysbench-0.4.12.16/sysbench/sysbench --test=fileio --num-threads=4 --file-total-size=512M --file-test-mode=rndrw run
n=$(( n + 1 ))
done
