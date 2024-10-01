#!/bin/bash
set -e;

trap 'echo "UPSD main process is terminated"; exit' SIGINT SIGABRT SIGTERM SIGQUIT SIGKILL;

init-config() {
    DUMMY_DEVICE_DIR="/nut_devices";
    NUT_CONF_PATH="/etc/nut/ups.conf";
    TEMP_NUT_CONF_PATH="/tmp/tmp-ups.conf";

    if [ -f "${TEMP_NUT_CONF_PATH}" ]; then
        rm "${TEMP_NUT_CONF_PATH}";
    fi

    for device in $(find "${DUMMY_DEVICE_DIR}" -type f -name '*.dev' -o -name '*.seq');
    do
        DEVICE_FILE=$(basename "${device}");
        DEVICE_NAME=$(basename "${DEVICE_FILE,,}" | awk '{print $1}');

        CONFIG=$(cat <<EOF
[${DEVICE_NAME}]
    driver = dummy-ups
    port = "${DEVICE_FILE}"
    desc = "dummy-ups ${DEVICE_NAME}"
EOF
        );

        echo "${CONFIG}\n" >> "${TEMP_NUT_CONF_PATH}"
        install "${device}" "/etc/nut/${DEVICE_FILE}";
    done;

    install "${TEMP_NUT_CONF_PATH}" "${NUT_CONF_PATH}";

    service nut-server restart;
    service nut-client restart;
    upsdrvctl stop;
    upsdrvctl start;
}

init-config;
sleep infinity;
