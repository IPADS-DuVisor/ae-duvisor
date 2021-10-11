echo "" > /var/log/mysql/error.log

mysqld &

ready="NOT A LEGAL STRING"

while [[ $ready !=  *"ready for connections"* ]]
do
	ready=`tail -1 /var/log/mysql/error.log`
done
