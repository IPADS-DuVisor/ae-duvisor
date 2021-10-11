kill -9 `ps -ef | awk '$8=="/usr/bin/httpd" {print $2}'`

ps -ef | grep 'mdev' | grep -v grep | awk '{print $2}' | xargs -r kill -9

ps -ef | grep 'logd' | grep -v grep | awk '{print $2}' | xargs -r kill -9

ps -ef | grep 'dropbear' | grep -v grep | awk '{print $2}' | xargs -r kill -9

#kill -9 `ps -ef | awk '$8=="/usr/sbin/apache2" {print $2}'`

