kill -9 `ps -ef | awk '$8=="/usr/local/bin/memcached" {print $2}'`
