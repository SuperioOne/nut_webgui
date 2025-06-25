# Custom build configuration

By default, build pipeline *(aka shell scripts glued together)* uses [`build.config.json`](build.config.json) to 
generate dockerfiles and build binaries. You can provide custom configuration to create custom build pipeline.

## Example: Alpine Linux as base image with ARM only builds

1. Create a new custom config file.

    ```json
    {
      "binary": [
        "aarch64-musl",
        "armv7-musleabi",
        "armv6-musleabi"
      ],
      "oci": {
        "manifest": {
          "name": "$MY_MANIFEST_NAME",
          "tags": [
            "$VERSION",
            "latest"
          ]
        },
        "images": [
          {
            "target": "aarch64-musl",
            "base_image": "docker.io/alpine:latest",
            "platform": "linux/arm64/v8",
            "os": "linux",
            "arch": "arm64",
            "variant": "v8",
            "tags": [
              "$VERSION-arm64"
            ]
          },
          {
            "target": "armv6-musleabi",
            "base_image": "docker.io/alpine:latest",
            "platform": "linux/arm/v6",
            "os": "linux",
            "arch": "arm",
            "variant": "v6",
            "tags": [
              "$VERSION-armv6"
            ]
          },
          {
            "target": "armv7-musleabi",
            "base_image": "docker.io/alpine:latest",
            "platform": "linux/arm/v7",
            "os": "linux",
            "arch": "arm",
            "variant": "v7",
            "tags": [
              "$VERSION-armv7"
            ]
          }
        ]
      }
    }
    ```

2. (Optional) Create custom docker template

    The following environment variables are automatically populated from config file:
    - `$BASE_CONTAINER_IMAGE`
    - `$PLATFORM`
    - `$EXE_DIR`

    example dockerfile template with custom username:
    ```dockerfile
    FROM --platform=${PLATFORM} ${BASE_CONTAINER_IMAGE}
    RUN adduser -H -D -g "<${MY_CUSTOM_USER}>" "${MY_CUSTOM_USER}"; \
            install -d -m 774 -o root -g "${MY_CUSTOM_USER}" /etc/nut_webgui
    COPY --chmod=750 --chown=root:${MY_CUSTOM_USER} ${EXE_DIR}/nut_webgui /usr/local/bin/nut_webgui
    COPY --chmod=754 --chown=root:${MY_CUSTOM_USER} ./containers/server_start.sh /usr/local/bin/nut_webgui_server.sh
    COPY --chmod=774 --chown=root:${MY_CUSTOM_USER} ./containers/config.toml /etc/nut_webgui/config.toml
    COPY --chmod=774 --chown=root:${MY_CUSTOM_USER} ./containers/config.toml /usr/local/share/nut_webgui/config.toml
    USER ${MY_CUSTOM_USER}
    CMD ["/usr/local/bin/nut_webgui_server.sh"]
    ```

3. Build and generate everything

    ```bash
    # All custom variables populated via GNU's `envsubst` text tool.
    export MY_CUSTOM_USER="overlord";
    export MY_MANIFEST_NAME="nut_custom";

    # Builds binaries defined in .binary[] section
    make build-all  BUILD_CONFIG="/path/of/custom.config.json";

    # Generates dockerfiles from .oci.images[] section
    make gen-dockerfiles  BUILD_CONFIG="/path/of/custom.config.json" DOCKER_TEMPLATE="/path/of/custom.docker.template";
    ```

4. (Optional, requires buildah) Build images and manifest locally with [`tools/build_images.sh`](tools/build_images.sh)

    ```bash
    # if your buildah requires sudo access, use it with `sudo -E`

    ./tools/build_images.sh \
      -c "/path/of/custom.config.json" \
      -a "bin/dockerfiles/annotations.json" \
      -f "bin/dockerfiles"

    # Help menu for other options like pushing images to registry, dry-run etc
    # ./tools/build_images.sh -h
    ```
