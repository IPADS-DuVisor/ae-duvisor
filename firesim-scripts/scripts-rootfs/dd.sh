TIMES=${TIMES:-3}

n=0

while [ $n -lt $TIMES ]; do 
    dd if=/dev/zero of=test.img bs=1M count=100 
    n=$(( n + 1 ))
done

exit 0
