rm -r bk-7.3.3

TIMES=${TIMES:-3}

n=0

while [ $n -lt $TIMES ]; do 
    time  tar -mxzf bk-7.3.3.src.tar.gz
n=$(( n + 1 ))
done
