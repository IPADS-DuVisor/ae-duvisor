sysbench --test=oltp \
       	--mysql-host=localhost \
      	--mysql-port=3306\
       	--mysql-user=root \
       	--mysql-password='ipads123' \
	--mysql-db=tssysbench \
	--db-driver=mysql \
	--oltp-num-tables=3 \
	--oltp-table-size=500000  \
	--report-interval=10 \
	prepare
