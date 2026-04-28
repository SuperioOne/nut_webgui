FROM docker.io/ubuntu:devel
COPY --chmod=777 setup.sh /tmp/setup.sh
COPY --chmod=744 ./cert_db /usr/local/ups/etc/cert_db
COPY --chmod=744 ./test_devices/ /nut_devices
COPY --chmod=777 upsd_init.sh /usr/bin/upsd_init.sh
RUN /tmp/setup.sh
CMD ["/usr/bin/upsd_init.sh"]
