#!/bin/bash
set -e;

export DEBIAN_FRONTEND=noninteractive
export TZ=Etc/UTC
apt-get update -y
apt-get upgrade -y
apt-get install -y nut nut-client nut-server
apt-get clean

# Binds server port to 0.0.0.0
echo "Configuring NUT daemon file..."
echo "LISTEN 0.0.0.0 3493" > /etc/nut/upsd.conf
cat /etc/nut/upsd.conf

echo "Creating default NUT user..."
UPSD_USER=$(cat <<EOF
[admin]
        password = test
        instcmds = all
        upsmon primary
EOF
);

echo "${UPSD_USER}" > /etc/nut/upsd.users
cat /etc/nut/upsd.users

echo "Enabling net server..."
echo "MODE=netserver" > /etc/nut/nut.conf
cat /etc/nut/nut.conf

