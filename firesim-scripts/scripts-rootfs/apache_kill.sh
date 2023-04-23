#kill -9 `ps -ef | awk '$8=="/usr/bin/httpd" {print $2}'`

kill -9 `ps -ef | awk '$8=="/usr/sbin/apache2" {print $2}'`

