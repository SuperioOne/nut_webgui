#!/bin/bash
set -e;

export DEBIAN_FRONTEND=noninteractive
export TZ=Etc/UTC
apt-get update -y
apt-get upgrade -y
apt-get install -y nut nut-client nut-server

# Binds server port to 0.0.0.0
cat /etc/nut/upsd.conf | sed 's/127\.0\.0\.1/0.0.0.0/g' > /etc/nut/upsd.conf

# Setup default user
UPSD_USER=$(cat <<EOF
[admin]
        password = test
        instcmds = all
        upsmon primary
EOF
);

echo "${UPSD_USER}" >> /etc/nut/upsd.users

# Enable netserver
echo "MODE=netserver" > /etc/nut/nut.conf
