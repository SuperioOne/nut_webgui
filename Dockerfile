# STAGE: Web component and style builder
ARG TARGETPLATFORM
FROM --platform=$TARGETPLATFORM node:latest as UI_BUILDER
RUN npm install -g pnpm
WORKDIR /build_dir
COPY ./package.json ./pnpm-lock.yaml ./style.css ./tailwind.config.js ./
COPY ./web_components/package.json ./web_components/pnpm-lock.yaml ./web_components/
RUN pnpm install -r
COPY ./server/src ./server/src
COPY ./web_components/src ./web_components/src
RUN pnpm run build:release && pnpm run -C ./web_components build:release

# STAGE: Server builder
ARG TARGETPLATFORM
FROM --platform=$TARGETPLATFORM rust:latest as SERVER_BUILDER
ARG RUST_TOOLCHAIN
RUN apt-get update && \
    apt-get -y install ca-certificates cmake musl-tools libssl-dev openssl gcc-aarch64-linux-gnu clang llvm libc6-dev-arm64-cross && \
    rustup target add "$RUST_TOOLCHAIN"
WORKDIR /build_dir
COPY ./server/Cargo.toml ./server/Cargo.lock ./server/askama.toml ./
COPY ./server/src ./src
COPY ./server/.cargo ./.cargo
RUN cargo build --target=$RUST_TOOLCHAIN -r

# STAGE: Main image
FROM --platform=$TARGETPLATFORM alpine:latest
ARG RUST_TOOLCHAIN
RUN adduser -H -D -g "<nut_web>" nut_webgui
COPY --chmod=750 --chown=root:nut_webgui --from=SERVER_BUILDER "./build_dir/target/$RUST_TOOLCHAIN/release/nut_webgui" /opt/nut_webgui/nut_webgui
COPY --chmod=750 --chown=root:nut_webgui ./server_start.sh /opt/nut_webgui/server_start.sh
COPY --chmod=744 --chown=root:nut_webgui --from=UI_BUILDER ./build_dir/dist/static /opt/nut_webgui/static/
WORKDIR /opt/nut_webgui
USER nut_webgui
CMD ["/opt/nut_webgui/server_start.sh"]