#!/bin/bash
set -e;

# ANSI colors
RST='\033[0m'      # reset
C_G='\033[0;32m'   # color green
C_R='\033[0;31m'   # color red
C_Y='\033[0;33m'   # color yellow
C_Cy='\033[0;36m'  # color cyan

function p_info {
    echo -e "$C_Cy$@$RST"
}

function p_err {
    echo -e "$C_R$@$RST"
}

function p_warn {
    echo -e "$C_Y$@$RST"
}

function p_success {
    echo -e "$C_G$@$RST"
}

OCI_IMAGE_NAMESPACE="org.opencontainers.image";
CREATED_IMAGES="";

ARG_ANNOTATIONS="";
ARG_CONFIG_FILE="";
ARG_DRY_RUN="false";
ARG_PUBLISH="false";
ARG_REGISTRY="";
ARG_TLS_VERIFY="false";
ARG_USER_NAME="";
ARG_USER_PASS="";
ARG_DOCKERFILES_DIR="";

while getopts "hdtu:p:a:c:r:f:" opt; do
    case $opt in
        "d")
            ARG_DRY_RUN="true";
            ;;
        "r")
            ARG_PUBLISH="true";
            ARG_REGISTRY="$OPTARG";
            if [[ -z "$ARG_REGISTRY" ]]; then
                p_err "Error: Registry argument is empty";
                exit 1;
            fi
            ;;
        "t")
            ARG_TLS_VERIFY="true";
            ;;
        "u")
            ARG_USER_NAME="$OPTARG";
            ;;
        "p")
            ARG_USER_PASS="$OPTARG";
            ;;
        "f")
            ARG_DOCKERFILES_DIR="$OPTARG";
            ;;
        "a")
            ARG_ANNOTATIONS="$OPTARG";
            ;;
        "c")
            ARG_CONFIG_FILE="$OPTARG";
            ;;
        ?)
            printf "Usage %s: -a <annotation-file> -c <config-file> -f <dockerfiles-dir> [-h] [-d] [-r <registry>] [-t] [-u <username>] [-p <password>]\n" $0
            echo ""
            echo "Required Arguments:"
            echo "  -a <annotation-file> : Required. OCI annotations json file."
            echo "  -c <config-file>     : Required. Path to the build configuration file."
            echo "  -f <dockerfile dir>  : Required. Path to dockerfile directory."
            echo ""
            echo "Optional Arguments:"
            echo "  -d                   : Dry-run mode (no image building/publishing)."
            echo "  -h                   : Show this help message."
            echo "  -p <password>        : User password for authentication."
            echo "  -r <registry-url>    : Push images and manifest to the target registry."
            echo "  -t                   : Enable TLS verification when pushing. Default is false."
            echo "  -u <username>        : Registry username for authentication."
            exit 2;
            ;;
    esac;
done;

if [ "$ARG_DRY_RUN" == "true" ]; then
    p_info "Dry run enabled."
fi

if [ "$ARG_TLS_VERIFY" != "true" ]; then
    p_warn "Warning! TLS verification is disabled."
fi

if [ "$ARG_PUBLISH" != "true" ]; then
    p_info "Local build only. Images won't be published to the container registry."
fi

if [ ! -e "$ARG_ANNOTATIONS" ]; then
    p_err "Annotation config '$ARG_ANNOTATIONS' does not exits.";
    exit 1;
else
    OCI_AUTHORS="$OCI_IMAGE_NAMESPACE.authors=$(jq -r '.authors' "$ARG_ANNOTATIONS")";
    OCI_URL="$OCI_IMAGE_NAMESPACE.url=$(jq -r '.url' "$ARG_ANNOTATIONS")";
    OCI_DOCUMENTATION="$OCI_IMAGE_NAMESPACE.documentation=$(jq -r '.documentation' "$ARG_ANNOTATIONS")";
    OCI_SOURCE="$OCI_IMAGE_NAMESPACE.source=$(jq -r '.source' "$ARG_ANNOTATIONS")";
    OCI_VERSION="$OCI_IMAGE_NAMESPACE.version=$(jq -r '.version' "$ARG_ANNOTATIONS")";
    OCI_REVISION="$OCI_IMAGE_NAMESPACE.revision=$(jq -r '.revision' "$ARG_ANNOTATIONS")";
    OCI_LICENSES="$OCI_IMAGE_NAMESPACE.licenses=$(jq -r '.licenses' "$ARG_ANNOTATIONS")";
    OCI_TITLE="$OCI_IMAGE_NAMESPACE.title=$(jq -r '.title' "$ARG_ANNOTATIONS")";
    OCI_DESCRIPTION="$OCI_IMAGE_NAMESPACE.description=$(jq -r '.description' "$ARG_ANNOTATIONS")";

    export PROJECT_NAME="$(jq -r '.title' "$ARG_ANNOTATIONS")";
    export VERSION="$(jq -r '.version' "$ARG_ANNOTATIONS")";
    export VERSION_MAJOR_MINOR="$(echo "$VERSION" | awk -F '.' '{print $1"."$2}')";

    if [ -z "$PROJECT_NAME" ]; then
        p_err "Project name is empty, something is wrong with annotations.json file";
        exit 1;
    fi

    if [ -z "$VERSION" ]; then
        p_err "Version information is empty, check $ARG_ANNOTATIONS file";
        exit 1;
    fi

    if [ -z "$VERSION_MAJOR_MINOR" ]; then
        p_err "Version MAJOR.MINOR is empty";
        exit 1;
    fi
fi

if [ ! -e "$ARG_CONFIG_FILE" ]; then
    p_err "Build config '$ARG_CONFIG_FILE' does not exits.";
    exit 1;
fi

if [ ! -d "$ARG_DOCKERFILES_DIR" ]; then
    p_err "Dockerfile directory '$ARG_DOCKERFILES_DIR' does not exits.";
    exit 1;
fi


CONFIG_FILE_CONTENT="$(cat "$ARG_CONFIG_FILE" | envsubst)";
MANIFEST_IMAGES=();
MANIFEST_NAME="$(echo "$CONFIG_FILE_CONTENT" | jq -r '.oci.manifest.name')";

for conf in $(echo "$CONFIG_FILE_CONTENT" | jq -c '.oci.images[]'); do
    _TARGET="$(echo "$conf" | jq -r '.target')";
    _TARGET_FILE="$ARG_DOCKERFILES_DIR/$_TARGET.Dockerfile";
    _LOCAL_IMAGE_NAME="$MANIFEST_NAME:${_TARGET}";
    _ARCH_VARIANT="$(echo "$conf" | jq -r '.variant')";

    if [[ -z "$_TARGET" && ! -f "$_TARGET_FILE" ]]; then
        p_err "Invalid target dockerfile '$_TARGET_FILE'";
        exit 1;
    fi

    p_info "Target file $_TARGET_FILE";
    p_info "Building image $_LOCAL_IMAGE_NAME";

    if [ "$ARG_DRY_RUN" != "true" ]; then
        buildah build \
            --layers \
            --variant "$_ARCH_VARIANT" \
            --annotation "$OCI_VERSION" \
            --annotation "$OCI_AUTHORS" \
            --annotation "$OCI_URL" \
            --annotation "$OCI_DOCUMENTATION" \
            --annotation "$OCI_SOURCE" \
            --annotation "$OCI_VERSION" \
            --annotation "$OCI_REVISION" \
            --annotation "$OCI_LICENSES" \
            --annotation "$OCI_TITLE" \
            --annotation "$OCI_DESCRIPTION" \
            -t "$_LOCAL_IMAGE_NAME" \
            -f "$_TARGET_FILE"
    fi

    p_success "Build completed: $_LOCAL_IMAGE_NAME";
done;

_MANIFEST_NAME="$MANIFEST_NAME:local";
p_info "Creating manifest: $_MANIFEST_NAME";

if [ "$ARG_DRY_RUN" != "true" ]; then
    if buildah manifest exists "$_MANIFEST_NAME"; then
        p_warn "$_MANIFEST_NAME already exists, removing existing manifest from local.";
        buildah manifest rm "$_MANIFEST_NAME"
        p_success "Removed $_MANIFEST_NAME";
    fi

    buildah manifest create \
        --annotation "$OCI_VERSION" \
        --annotation "$OCI_AUTHORS" \
        --annotation "$OCI_URL" \
        --annotation "$OCI_DOCUMENTATION" \
        --annotation "$OCI_SOURCE" \
        --annotation "$OCI_VERSION" \
        --annotation "$OCI_REVISION" \
        --annotation "$OCI_LICENSES" \
        --annotation "$OCI_TITLE" \
        --annotation "$OCI_DESCRIPTION" \
        "$_MANIFEST_NAME"
fi

for conf in $(echo "$CONFIG_FILE_CONTENT" | jq -c '.oci.images[]'); do
    _TARGET="$(echo "$conf" | jq -r '.target')";
    _VARIANT="$(echo "$conf" | jq -r '.variant')";
    _LOCAL_IMAGE_NAME="$MANIFEST_NAME:$_TARGET"

    if [ "$ARG_DRY_RUN" != "true" ];then
        buildah manifest add --variant "$_VARIANT" "$_MANIFEST_NAME" "$_LOCAL_IMAGE_NAME";
    fi

    p_success "$_LOCAL_IMAGE_NAME added to $_MANIFEST_NAME";
done;

if [ "$ARG_PUBLISH" == "true" ];then
    for tag in $(echo "$CONFIG_FILE_CONTENT" | jq -cr '.oci.manifest.tags[]'); do
        if [ "$ARG_DRY_RUN" != "true" ]; then
            buildah manifest push \
                --tls-verify="$ARG_TLS_VERIFY" \
                --creds "$ARG_USER_NAME:$ARG_USER_PASS" \
                --all "$_MANIFEST_NAME" "docker://$ARG_REGISTRY/$MANIFEST_NAME:$tag";
        fi

        p_success "$_MANIFEST_NAME pushed to the $ARG_REGISTRY/$MANIFEST_NAME:$tag"
    done;
fi

if [ "$ARG_PUBLISH" == "true" ];then
    for conf in $(echo "$CONFIG_FILE_CONTENT" | jq -c '.oci.images[]'); do
        _TARGET="$(echo "$conf" | jq -r '.target')";
        _LOCAL_IMAGE_NAME="$MANIFEST_NAME:${_TARGET}";

        for alias_tag in $(echo "$conf" | jq -r '.tags[]');do
            _IMAGE_NAME="$MANIFEST_NAME:$alias_tag";

            if [ "$ARG_DRY_RUN" != "true" ]; then
                buildah push \
                    --tls-verify="$ARG_TLS_VERIFY" \
                    --creds "$ARG_USER_NAME:$ARG_USER_PASS" \
                    "$_LOCAL_IMAGE_NAME" "docker://$ARG_REGISTRY/$_IMAGE_NAME";
            fi

            p_success "Tagged $_LOCAL_IMAGE_NAME as $ARG_REGISTRY/$_IMAGE_NAME";
        done;
    done;
fi

p_success "Task Completed!"
