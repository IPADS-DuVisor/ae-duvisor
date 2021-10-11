TIMES=${TIMES:-3}

n=0

while [ $n -lt $TIMES ]; do 
    hackbench
n=$(( n + 1 ))
done
