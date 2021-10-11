kill -9 `ps -ef | awk '$8=="mysqld" {print $2}'`
