# STAGE: Web component and style builder
ARG TARGETPLATFORM
FROM --platform=${TARGETPLATFORM} docker.io/node:latest as UI_BUILDER
RUN npm install -g pnpm
WORKDIR /build_dir
COPY ./package.json ./pnpm-lock.yaml ./style.css ./tailwind.config.js ./
COPY client/package.json ./web_components/pnpm-lock.yaml ./web_components/
RUN pnpm install -r
COPY ./server/src ./server/src
COPY client/src ./web_components/src
RUN pnpm run build:release && pnpm run -C ./client build:release
COPY ./icon.svg ./dist/static/icon.svg

# STAGE: Server builder
ARG TARGETPLATFORM
FROM --platform=${TARGETPLATFORM} docker.io/rust:latest as SERVER_BUILDER
ARG RUST_TOOLCHAIN
RUN apt-get update && \
    apt-get -y install ca-certificates cmake musl-tools libssl-dev openssl gcc-aarch64-linux-gnu clang llvm libc6-dev-arm64-cross && \
    rustup target add "$RUST_TOOLCHAIN"
WORKDIR /build_dir
COPY ./server/Cargo.toml ./server/Cargo.lock ./server/askama.toml ./
COPY ./server/src ./src
COPY ./server/.cargo ./.cargo
RUN cargo fetch --target "$RUST_TOOLCHAIN"
RUN mkdir /build_dir/output && \
    cargo build --target "$RUST_TOOLCHAIN" --release && \
    cp "/build_dir/target/$RUST_TOOLCHAIN/release/nut_webgui" /build_dir/output

# STAGE: Main image
ARG TARGETPLATFORM
ARG VERSION_TAG
ARG ALPINE_TAG
LABEL org.opencontainers.image.authors="Timur Olur <pm@smdd.dev>"
LABEL org.opencontainers.image.version="${VERSION_TAG}"
LABEL org.opencontainers.image.source="https://github.com/SuperioOne/nut_webgui"
LABEL org.opencontainers.image.url="https://github.com/SuperioOne/nut_webgui"
LABEL org.opencontainers.image.documentation="https://raw.githubusercontent.com/SuperioOne/nut_webgui/master/README.md"
LABEL org.opencontainers.image.licenses="Apache-2.0"
LABEL org.opencontainers.image.title="NUT Web GUI"
LABEL org.opencontainers.image.description="Light weight web interface for Network UPS Tools."
LABEL org.opencontainers.image.vendor="Timur Olur"
LABEL org.opencontainers.image.base.name="docker.io/alpine:${ALPINE_TAG}"
FROM --platform=${TARGETPLATFORM} docker.io/alpine:${ALPINE_TAG}
RUN adduser -H -D -g "<nut_web>" nut_webgui
COPY --chmod=750 --chown=root:nut_webgui ./server_start.sh /opt/nut_webgui/server_start.sh
COPY --chmod=750 --chown=root:nut_webgui --from=SERVER_BUILDER ./build_dir/output/nut_webgui /opt/nut_webgui/nut_webgui
COPY --chmod=744 --chown=root:nut_webgui --from=UI_BUILDER ./build_dir/dist/static /opt/nut_webgui/static/
WORKDIR /opt/nut_webgui
USER nut_webgui
CMD ["/opt/nut_webgui/server_start.sh"]