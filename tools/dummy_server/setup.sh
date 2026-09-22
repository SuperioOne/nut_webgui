#!/bin/bash
set -e;

export DEBIAN_FRONTEND=noninteractive
export TZ=Etc/UTC
apt-get update -y
apt-get upgrade -y
apt-get install -y nut nut-client nut-server libnss3-tools
apt-get clean

echo "Configuring NUT daemon file..."
UPSD_CONF=$(cat <<EOF
LISTEN 0.0.0.0 3493
CERTPATH /usr/local/ups/etc/cert_db
CERTIDENT "nut server" "test"
EOF
);

echo "$UPSD_CONF" > /etc/nut/upsd.conf
cat /etc/nut/upsd.conf

echo "Creating default NUT user..."
UPSD_USER=$(cat <<EOF
[admin]
        password = test
        instcmds = all
        actions = set
        actions = fsd
        upsmon primary
EOF
);

echo "$UPSD_USER" > /etc/nut/upsd.users
cat /etc/nut/upsd.users

echo "Enabling net server..."
echo "MODE=netserver" > /etc/nut/nut.conf
cat /etc/nut/nut.conf

