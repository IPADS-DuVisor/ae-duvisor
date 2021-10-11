HOST=${HOST:-"http://127.0.0.1/data"}
TIMES=${TIMES:-3}

n=0

while [ $n -lt $TIMES ]; do 
    curl -O ${HOST}
    n=$(( n + 1 ))
done

exit 0
