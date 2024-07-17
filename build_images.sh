#!/usr/bin/bash
set -e;

# ANSI colors
RST='\033[0m'      # reset
C_G='\033[0;32m'   # color green
C_R='\033[0;31m'   # color red
C_Y='\033[0;33m'   # color yellow
C_Cy='\033[0;36m'  # color cyan

REGISTRY=""
TLS_VERIFY="false"
ALIAS_TAGS=""
DRY_RUN="false"
PUBLISH="false"
ANNOTATION_CONF="./containers/annotation.conf"

while getopts "hdtp:a:c:" opt; do
    case $opt in
        "d")
            DRY_RUN="true";
            ;;
        "p")
            PUBLISH="true";
            REGISTRY="${OPTARG}";
            if [[ -z "$REGISTRY" ]]; then
                echo -e "${C_R}Error: Registry argument is empty${RST}"
                exit 1
            fi
            ;;
        "t")
            TLS_VERIFY="true";
            ;;
        "a")
            ALIAS_TAGS="${OPTARG}";
            ;;
        "c")
            if [[ -e "${OPTARG}" ]]; then
                ANNOTATION_CONF="${OPTARG}";
            else
                echo -e "${C_R}Annotation config file does not exits.";
                exit 1;
            fi
            ;;
        "h")
            printf "Usage %s: [-h] [-d] [-p registry] [-t] [-a alias-tags].\n" $0
            echo "Arguments:"
            echo "  -h                 : Help menu."
            echo "  -d                 : Dry-run. Only prints messages."
            echo "  -p <registry-url>  : Pushes images and manifest to the target registry."
            echo "  -t                 : Enable TLS verification. Default is false."
            echo "  -a <tag1;tag2;...> : Additional alias tags separeted with semicolons (;)"
            exit 2;
            ;;
        ?)
            printf "Usage %s: [-h] [-d] [-p registry] [-t] [-a alias-tags].\nUse '%s --h' for flag details.\n" $0 $0
            exit 2;
            ;;
    esac;
done;

source "${ANNOTATION_CONF}";

if [ "$DRY_RUN" == "true" ]; then
    echo -e "${C_Cy}Info: Dry run enabled.${RST}"
fi

if [ "$TLS_VERIFY" != "true" ]; then
    echo -e "${C_Y}Warning: TLS verification disabled.${RST}"
fi

if [ "${PUBLISH}" != "true" ]; then
    echo -e "${C_Cy}Info: Local build only. Images won't be published to the container registry.${RST}"
fi

CREATED_IMAGES="";
DOCKER_FILES=$(find ./containers/ -type f -name *.Dockerfile);
OCI_IMAGE_NAMESPACE="org.opencontainers.image";

function find_arch_variant() {
    case "${1}" in
        amd64)
            echo ""
            ;;
        amd64-v3)
            echo "v3"
            ;;
        amd64-v4)
            echo "v4"
            ;;
        arm64)
            echo "v8"
            ;;
        armv7)
            echo "v7"
            ;;
        armv6)
            echo "v6"
            ;;
        riscv64)
            echo ""
            ;;
        *)
            echo -e "${C_R}Unknown architecture type ${1}.${RST}";
            exit 1
            ;;
    esac;
}

for TARGET_FILE in ${DOCKER_FILES}; do
    _ARCH=$(basename -s ".Dockerfile" "${TARGET_FILE}");
    _IMAGE_TAG="${NAME}:${VERSION}-${_ARCH}";
    _ARCH_VARIANT=$(find_arch_variant "${_ARCH}")

    echo -e "${C_Cy}Dockerfile ${TARGET_FILE}${RST}";
    echo -e "${C_Cy}Building ${_IMAGE_TAG}${RST}";

    test "${DRY_RUN}" != "true" && \
        buildah build \
        --layers \
        --variant "${_ARCH_VARIANT}" \
        --annotation "${OCI_IMAGE_NAMESPACE}.authors=${AUTHORS}" \
        --annotation "${OCI_IMAGE_NAMESPACE}.url=${HOME_URL}" \
        --annotation "${OCI_IMAGE_NAMESPACE}.documentation=${DOCUMENTATION}" \
        --annotation "${OCI_IMAGE_NAMESPACE}.source=${SOURCE}" \
        --annotation "${OCI_IMAGE_NAMESPACE}.version=${VERSION}" \
        --annotation "${OCI_IMAGE_NAMESPACE}.revision=${REVISION}" \
        --annotation "${OCI_IMAGE_NAMESPACE}.licenses=${LICENSES}" \
        --annotation "${OCI_IMAGE_NAMESPACE}.title=${NAME}" \
        --annotation "${OCI_IMAGE_NAMESPACE}.description=${DESCRIPTION}" \
        -t "${_IMAGE_TAG}" \
        -f "${TARGET_FILE}"

    echo -e "${C_G}Container image '${_IMAGE_TAG}' built.${RST}";
    CREATED_IMAGES+="${_IMAGE_TAG} ";
done;

MANIFEST_NAME="${NAME}:${VERSION}";

if [[ "${DRY_RUN}" != "true" ]]; then
    if buildah manifest exists "${MANIFEST_NAME}"; then
        echo -e "${C_R}${MANIFEST_NAME} already exists, remove it before proceeding.${RST}";
        buildah manifest rm "${MANIFEST_NAME}"
        echo -e "${C_Y}Removed ${MANIFEST_NAME}${RST}";
    fi

    echo -e "${C_Y}Creating manifest: ${MANIFEST_NAME}.${RST}";

    buildah manifest create \
        --annotation "${OCI_IMAGE_NAMESPACE}.authors=${AUTHORS}" \
        --annotation "${OCI_IMAGE_NAMESPACE}.url=${HOME_URL}" \
        --annotation "${OCI_IMAGE_NAMESPACE}.documentation=${DOCUMENTATION}" \
        --annotation "${OCI_IMAGE_NAMESPACE}.source=${SOURCE}" \
        --annotation "${OCI_IMAGE_NAMESPACE}.version=${VERSION}" \
        --annotation "${OCI_IMAGE_NAMESPACE}.revision=${REVISION}" \
        --annotation "${OCI_IMAGE_NAMESPACE}.licenses=${LICENSES}" \
        --annotation "${OCI_IMAGE_NAMESPACE}.title=${NAME}" \
        --annotation "${OCI_IMAGE_NAMESPACE}.description=${DESCRIPTION}" \
        "${MANIFEST_NAME}"
fi

for IMAGE in ${CREATED_IMAGES}; do
    _TAG_ARCH=$(awk "match(\$0,/^${NAME}:${VERSION}-(.*)/, matches) {print matches[1]}" <<< "${IMAGE}");
    _ARCH_VARIANT=$(find_arch_variant "${_TAG_ARCH}")

    test "${DRY_RUN}" != "true" && \
        buildah manifest add --variant "${_ARCH_VARIANT}" "${MANIFEST_NAME}" "${IMAGE}";

    echo -e "${C_Y}${IMAGE} added to ${MANIFEST_NAME}${RST}.";
done;

if [[ "$PUBLISH" == "true" ]]; then
    test "${DRY_RUN}" != "true" && \
        buildah login --tls-verify="${TLS_VERIFY}" "${REGISTRY}"

    TAG_LIST=("${VERSION}" ${ALIAS_TAGS[@]});
    for TAG in "${TAG_LIST[@]}";
    do
        if [ -n "${TAG}" ]; then
            echo -e "${C_Y}Pushing manifest '${MANIFEST_NAME}' as '${REGISTRY,,}/${NAME}:${TAG}'.${RST}"

            test "${DRY_RUN}" != "true" && \
                buildah manifest push --tls-verify="${TLS_VERIFY}" --all "${MANIFEST_NAME}" "docker://${REGISTRY,,}/${NAME}:${TAG}"

            echo -e "${C_G}Push '${MANIFEST_NAME}' completed.${RST}";
        fi
    done;
fi

echo -e "${C_G}Task Completed${RST}"
