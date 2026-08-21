#!/usr/bin/env sh
set -e

ENABLE_AARCH64_GNU=
ENABLE_AARCH64_MUSL=
ENABLE_ARMV6_MUSLEABI=
ENABLE_ARMV7_MUSLEABI=
ENABLE_RISCV_GNU=
ENABLE_RISCV_MUSL=
ENABLE_X86_64_GNU=
ENABLE_X86_64_MUSL=
ENABLE_X86_64_V3_GNU=
ENABLE_X86_64_V3_MUSL=
ENABLE_X86_64_V4_GNU=
ENABLE_X86_64_V4_MUSL=
ENABLE_CONTAINER_IMAGE=
GITHUB_RELEASE_TARGETS=
FORGEJO_RELEASE_TARGETS=
OCI_REGISTRIES=

verify_publish_target() {
    _ALIAS="$(echo "$1" | awk -F "=" '{print $1}')"
    _TARGET_URI="$(echo "$1" | awk -F "=" '{
                dlmtr = index($0,"=");
                if(dlmtr < 1) {
                        print ""
                } else {
                        print substr($0,dlmtr+1)
                }
        }')"

    if [ -z "$_ALIAS" ]; then
        echo "publish target alias cannot be empty: $1"
        exit 1
    fi

    if [ -z "$_TARGET_URI" ]; then
        echo "publish target URI cannot be empty: $1"
        exit 1
    fi
}

set_github_target() {
    case "$GITHUB_RELEASE_TARGETS" in
        *"$1"*)
            ;;
        *)
            if [ -z "$GITHUB_RELEASE_TARGETS" ]; then
                GITHUB_RELEASE_TARGETS="$1"
            else
                GITHUB_RELEASE_TARGETS="$1;$GITHUB_RELEASE_TARGETS"
            fi
            ;;
    esac
}

set_forgejo_target() {
    case "$FORGEJO_RELEASE_TARGETS" in
        *"$1"*)
            ;;
        *)
            if [ -z "$FORGEJO_RELEASE_TARGETS" ]; then
                FORGEJO_RELEASE_TARGETS="$1"
            else
                FORGEJO_RELEASE_TARGETS="$1;$FORGEJO_RELEASE_TARGETS"
            fi
            ;;
    esac
}

set_target_registry() {
    if [ -z "$1" ]; then
        echo "registry target cannot be empty"
        exit 1
    fi

    case "$OCI_REGISTRIES" in
        *"$1;"*)
            ;;
        *)
            if [ -z "$OCI_REGISTRIES" ]; then
                OCI_REGISTRIES="$1"
            else
                OCI_REGISTRIES="$1;$OCI_REGISTRIES"
            fi
            ;;
    esac
}

printif() {
    if [ "$1" = "true" ]; then
        echo "$2"
    fi
}

list_active_targets() {
    printif "$ENABLE_AARCH64_GNU" "aarch64-gnu"
    printif "$ENABLE_AARCH64_MUSL" "aarch64-musl"
    printif "$ENABLE_ARMV6_MUSLEABI" "armv6-musleabi"
    printif "$ENABLE_ARMV7_MUSLEABI" "armv7-musleabi"
    printif "$ENABLE_RISCV_GNU" "riscv64gc-gnu"
    printif "$ENABLE_RISCV_MUSL" "riscv64gc-musl"
    printif "$ENABLE_X86_64_GNU" "x86-64-gnu"
    printif "$ENABLE_X86_64_MUSL" "x86-64-musl"
    printif "$ENABLE_X86_64_V3_GNU" "x86-64-v3-gnu"
    printif "$ENABLE_X86_64_V3_MUSL" "x86-64-v3-musl"
    printif "$ENABLE_X86_64_V4_GNU" "x86-64-v4-gnu"
    printif "$ENABLE_X86_64_V4_MUSL" "x86-64-v4-musl"
}

list_active_image_targets() {
    printif "$ENABLE_AARCH64_MUSL" "arm64"
    printif "$ENABLE_ARMV6_MUSLEABI" "armv6"
    printif "$ENABLE_ARMV7_MUSLEABI" "armv7"
    printif "$ENABLE_RISCV_MUSL" "riscv64"
    printif "$ENABLE_X86_64_MUSL" "amd64"
    printif "$ENABLE_X86_64_V3_MUSL" "amd64-v3"
    printif "$ENABLE_X86_64_V4_MUSL" "amd64-v4"
}

print_help() {
    echo 'USAGE: ./configure [OPTIONS]
OPTIONS
  --enable-all
        Enable all supported CPU architectures.

  --enable-aarch64-musl
        Enable Aarch64 (arm64/v8) support with musl.

  --enable-aarch64-gnu
        Enable Aarch64 (arm64/v8) support with glibc.

  --enable-armv6-musleabi
        Enable ARMv6 (arm/v6) support with musl (soft-float).

  --enable-armv7-musleabi
        Enable ARMv7 (arm/v7) support with musl (soft-float).

  --enable-riscv64gc-musl
        Enable RISC-V (riscv64) support with musl.

  --enable-riscv64gc-gnu
        Enable RISC-V (riscv64) support with glibc.

  --enable-x86-64-musl
        Enable x86_64 (amd64) support with musl.

  --enable-x86-64-gnu
        Enable x86_64 (amd64) support with glibc.

  --enable-x86-64-v3-musl
        Enable x86_64-v3 (amd64/v3) support with musl.

  --enable-x86-64-v3-gnu
        Enable x86_64-v3 (amd64/v3) support with glibc.

  --enable-x86-64-v4-musl
        Enable x86_64-v4 (amd64/v4) support with musl.

  --enable-x86-64-v4-gnu
        Enable x86_64-v4 (amd64/v4) support with glibc.

  --enable-container-image
        Enable container image build only if enabled architectures supports it.

  --release-github <ALIAS>=<URI>
        Add a GitHub repo as a release target.
        ENV variables for credentials:
                - GH_<ALIAS>_USERNAME
                - GH_<ALIAS>_ACCESS_TOKEN

  --release-forgejo <ALIAS>=<URI>
        Add a Forgejo/Gitea repo as a release target.
        ENV variables for credentials:
                - FJ_<ALIAS>_USERNAME
                - FJ_<ALIAS>_ACCESS_TOKEN

  --image-registry <ALIAS>=<URI>
        Add an OCI registry to publish images.
        ENV variables for credentials:
                - OCI_<ALIAS>_USERNAME
                - OCI_<ALIAS>_ACCESS_TOKEN

  -h, --help
        Show help menu.
'
}

while [ "$#" -gt 0 ];
do
    case "$1" in
        --enable-all)
            ENABLE_AARCH64_GNU=true
            ENABLE_AARCH64_MUSL=true
            ENABLE_ARMV6_MUSLEABI=true
            ENABLE_ARMV7_MUSLEABI=true
            ENABLE_RISCV_GNU=true
            ENABLE_RISCV_MUSL=true
            ENABLE_X86_64_GNU=true
            ENABLE_X86_64_MUSL=true
            ENABLE_X86_64_V3_GNU=true
            ENABLE_X86_64_V3_MUSL=true
            ENABLE_X86_64_V4_GNU=true
            ENABLE_X86_64_V4_MUSL=true
            ;;
        --enable-aarch64-musl)
            ENABLE_AARCH64_MUSL=true
            ;;
        --enable-aarch64-gnu)
            ENABLE_AARCH64_GNU=true
            ;;
        --enable-armv6-musleabi)
            ENABLE_ARMV6_MUSLEABI=true
            ;;
        --enable-armv7-musleabi)
            ENABLE_ARMV7_MUSLEABI=true
            ;;
        --enable-riscv64gc-musl)
            ENABLE_RISCV_MUSL=true
            ;;
        --enable-riscv64gc-gnu)
            ENABLE_RISCV_GNU=true
            ;;
        --enable-x86-64-musl)
            ENABLE_X86_64_MUSL=true
            ;;
        --enable-x86-64-gnu)
            ENABLE_X86_64_GNU=true
            ;;
        --enable-x86-64-v3-musl)
            ENABLE_X86_64_V3_MUSL=true
            ;;
        --enable-x86-64-v3-gnu)
            ENABLE_X86_64_V3_GNU=true
            ;;
        --enable-x86-64-v4-musl)
            ENABLE_X86_64_V4_MUSL=true
            ;;
        --enable-x86-64-v4-gnu)
            ENABLE_X86_64_V4_GNU=true
            ;;
        --enable-container-image)
            ENABLE_CONTAINER_IMAGE=true
            ;;
        --release-github)
            shift
            verify_publish_target "$1"
            set_github_target "$1"
            ;;
        --release-forgejo)
            shift
            verify_publish_target "$1"
            set_forgejo_target "$1"
            ;;
        --image-registry)
            shift
            verify_publish_target "$1"
            set_target_registry "$1"
            ;;
        --help)
            print_help
            exit 1
            ;;
        -h)
            print_help
            exit 1
            ;;
        *)
            echo "Unknown argument: $1"
            print_help
            exit 1
            ;;
    esac
    shift
done

METADATA="$(cargo metadata --no-deps --frozen --format-version 1 | jq -r '.packages[] |
        select(.name == "nut_webgui") |
        {
                title:.name,
                version:.version,
                url:.homepage,
                licenses:.license,
                documentation:.documentation,
                source:.repository,
                description:.description,
                authors:(.authors | join(";"))
        }')"

ANNOTATION_TITLE="$(echo "$METADATA" | jq -r '.title')"
ANNOTATION_VERSION="$(echo "$METADATA" | jq -r '.version')"
ANNOTATION_URL="$(echo "$METADATA" | jq -r '.url')"
ANNOTATION_LICENSES="$(echo "$METADATA" | jq -r '.licenses')"
ANNOTATION_DOCUMENTATION="$(echo "$METADATA" | jq -r '.documentation')"
ANNOTATION_SOURCE="$(echo "$METADATA" | jq -r '.source')"
ANNOTATION_DESCRIPTION="$(echo "$METADATA" | jq -r '.description')"
ANNOTATION_AUTHORS="$(echo "$METADATA" | jq -r '.authors')"
ANNOTATION_REVISION="$(git rev-parse --verify HEAD)"
VERSION="$ANNOTATION_VERSION"
VERSION_MAJOR="$(echo "$VERSION" | awk -F '.' '{if($1 == "0") { print $2 } else { print $1 }}')"
VERSION_MINOR="$(echo "$VERSION" | awk -F '.' '{if($1 == "0") { print $3 } else { print $2 }}')"
TARGETS=
IMAGE_TARGETS=

for target in $(list_active_targets); do
    TARGETS="$target $TARGETS"
done

for image in $(list_active_image_targets); do
    IMAGE_TARGETS="$image $IMAGE_TARGETS"
done

CONFIG_FILE="$(cat << EOF
ENABLE_AARCH64_GNU       := $ENABLE_AARCH64_GNU
ENABLE_AARCH64_MUSL      := $ENABLE_AARCH64_MUSL
ENABLE_ARMV6_MUSLEABI    := $ENABLE_ARMV6_MUSLEABI
ENABLE_ARMV7_MUSLEABI    := $ENABLE_ARMV7_MUSLEABI
ENABLE_RISCV_GNU         := $ENABLE_RISCV_GNU
ENABLE_RISCV_MUSL        := $ENABLE_RISCV_MUSL
ENABLE_X86_64_GNU        := $ENABLE_X86_64_GNU
ENABLE_X86_64_MUSL       := $ENABLE_X86_64_MUSL
ENABLE_X86_64_V3_GNU     := $ENABLE_X86_64_V3_GNU
ENABLE_X86_64_V3_MUSL    := $ENABLE_X86_64_V3_MUSL
ENABLE_X86_64_V4_GNU     := $ENABLE_X86_64_V4_GNU
ENABLE_X86_64_V4_MUSL    := $ENABLE_X86_64_V4_MUSL
ENABLE_CONTAINER_IMAGE   := $ENABLE_CONTAINER_IMAGE
FORGEJO_RELEASE_TARGETS  := $FORGEJO_RELEASE_TARGETS
GITHUB_RELEASE_TARGETS   := $GITHUB_RELEASE_TARGETS
OCI_REGISTRIES           := $OCI_REGISTRIES
ANNOTATION_AUTHORS       := $ANNOTATION_AUTHORS
ANNOTATION_DESCRIPTION   := $ANNOTATION_DESCRIPTION
ANNOTATION_DOCUMENTATION := $ANNOTATION_DOCUMENTATION
ANNOTATION_LICENSES      := $ANNOTATION_LICENSES
ANNOTATION_REVISION      := $ANNOTATION_REVISION
ANNOTATION_SOURCE        := $ANNOTATION_SOURCE
ANNOTATION_TITLE         := $ANNOTATION_TITLE
ANNOTATION_URL           := $ANNOTATION_URL
ANNOTATION_VERSION       := $ANNOTATION_VERSION
VERSION                  := $VERSION
VERSION_MAJOR            := $VERSION_MAJOR
VERSION_MINOR            := $VERSION_MINOR
TARGETS                  := $TARGETS
IMAGE_TARGETS            := $IMAGE_TARGETS
EOF
)"

echo "CONFIG SUMMARY"
echo "------------------------------------------"
echo "$CONFIG_FILE" | tee .config.mk
