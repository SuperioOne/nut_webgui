FROM docker.io/ubuntu:latest
COPY --chmod=777 setup.sh /tmp/setup.sh
RUN /tmp/setup.sh
COPY --chmod=777 upsd_init.sh /usr/bin/upsd_init.sh
CMD ["/usr/bin/upsd_init.sh"]
