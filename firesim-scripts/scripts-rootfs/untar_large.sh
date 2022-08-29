rm -r bk-7.3.3

TIMES=${TIMES:-3}

n=0

while [ $n -lt $TIMES ]; do
    ./readtime
   # time -p tar -mxzf bk-7.3.3.src.tar.gz
    #tar -mxzf bk-7.3.3.src.tar.gz
    /hackbench 5
    ./readtime
    echo "Untar finish"
    sync
    rm -r bk-7.3.3
n=$(( n + 1 ))
done
