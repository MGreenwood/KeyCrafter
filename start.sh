#!/bin/bash

# Create required nginx files
touch /var/log/nginx/access.log /var/log/nginx/error.log
chown www-data:www-data /var/log/nginx/access.log /var/log/nginx/error.log

# Start SSH daemon
/usr/sbin/sshd -D -e &

# Start nginx in the foreground
nginx -g 'daemon off;' 