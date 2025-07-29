PROJECT_NAME     := $(shell cargo metadata --no-deps --offline --format-version 1 | jq -r ".packages[0].name")
PROJECT_VER      := $(shell cargo metadata --no-deps --offline --format-version 1 | jq -r ".packages[0].version")
BIN_DIR          := ./bin
NODE_MODULES_DIR := ./nut_webgui_client/node_modules
DOCKER_TEMPLATE  := ./containers/Dockerfile.template
BUILD_CONFIG     := ./build.config.json
PROJECT_SRCS     := $(shell find . -type f -iregex "\./nut_webgui[^/]*/src/.*") \
					$(shell find . -type f -iname Cargo.toml) \
					$(shell find . -type f -iname Cargo.lock) \
					./nut_webgui_client/package.json
DIST_DIR         := $(BIN_DIR)/dist
BINARY_TARGETS   := $(shell jq -r '.binary[]' "$(BUILD_CONFIG)")

fn_output_path    = $(BIN_DIR)/$(1)/$(PROJECT_NAME)
fn_target_path    = target/$(1)/$(PROJECT_NAME)

.PHONY: help
help:
	@echo "RECEIPES"
	@echo "  build                : Generates server binary for the current system's CPU architecture and OS."
	@echo "  build-all            : Cross compiles everything."
	@echo "  clean                : Clears all build directories."
	@echo "  gen-dockerfiles      : Generates dockerfiles for all supported architectures."
	@echo "  init                 : Initializes project dependencies by calling package managers like pnpm."
	@echo "  pack                 : Compresses (.tar.gz) all compiled targets under the bin directory."
	@echo "  test                 : Calls test suites."
	@echo ""
	@echo "Specific build targets:"
	@echo "  build-aarch64-gnu    : linux/arm64/v8, links with glibc."
	@echo "  build-aarch64-musl   : linux/arm64/v8, self contained with musl."
	@echo "  build-armv6-musleabi : linux/arm/v6, self contained with musl, soft-floats."
	@echo "  build-armv7-musleabi : linux/arm/v7, self contained with musl, soft-floats."
	@echo "  build-riscv64gc-gnu  : linux/riscv64, links with glibc."
	@echo "  build-x86-64-gnu     : linux/amd64, links with glibc."
	@echo "  build-x86-64-musl    : linux/amd64, self contained with musl."
	@echo "  build-x86-64-v3-gnu  : linux/amd64/v3, links with glibc."
	@echo "  build-x86-64-v3-musl : linux/amd64/v3, self contained with musl."
	@echo "  build-x86-64-v4-gnu  : linux/amd64/v4, links with glibc."
	@echo "  build-x86-64-v4-musl : linux/amd64/v4, self contained with musl."

.PHONY: init
init:
	@if [ $$(which pnpm 2>/dev/null) ]; then \
		pnpm install -C ./nut_webgui_client; \
	elif [ $$(which npm 2>/dev/null) ]; then \
		npm install -C ./nut_webgui_client; \
	else \
		echo "System doesn't have pnpm or npm. Install at least one of them to initialize node_modules."; \
	fi

# Default toolchain

.PHONY: build
build: init $(PROJECT_SRCS)
	@echo "Building binaries for the current system's architecture."
	@cargo build -p nut_webgui --release
	@install -D $(call fn_target_path,release) $(call fn_output_path,release)

# x86-64 MUSL

.PHONY: build-x86-64-musl
build-x86-64-musl: init $(call fn_output_path,x86-64-musl)

$(call fn_output_path,x86-64-musl) &: $(PROJECT_SRCS)
	@echo "Building for x86_64-unknown-linux-musl"
	@export RUSTFLAGS="-Clink-self-contained=yes -Clinker=rust-lld" && \
		cargo build -p nut_webgui --target=x86_64-unknown-linux-musl --release
	@install -D $(call fn_target_path,x86_64-unknown-linux-musl/release) \
		$(call fn_output_path,x86-64-musl)

# x86-64-v3 MUSL

.PHONY: build-x86-64-v3-musl
build-x86-64-v3-musl: init $(call fn_output_path,x86-64-v3-musl)

$(call fn_output_path,x86-64-v3-musl) &: $(PROJECT_SRCS)
	@echo "Building for x86_64-v3-unknown-linux-musl"
	@export RUSTFLAGS="-Clink-self-contained=yes -Ctarget-cpu=x86-64-v3 -Clinker=rust-lld" && \
		cargo build -p nut_webgui --target=x86_64-unknown-linux-musl --release
	@install -D $(call fn_target_path,x86_64-unknown-linux-musl/release) \
		$(call fn_output_path,x86-64-v3-musl)

# x86-64-v4 MUSL

.PHONY: build-x86-64-v4-musl
build-x86-64-v4-musl: init $(call fn_output_path,x86-64-v4-musl)

$(call fn_output_path,x86-64-v4-musl) &: $(PROJECT_SRCS)
	@echo "Building for x86_64-v4-unknown-linux-musl"
	@export RUSTFLAGS="-Clink-self-contained=yes -Ctarget-cpu=x86-64-v4 -Clinker=rust-lld" && \
		cargo build -p nut_webgui --target=x86_64-unknown-linux-musl --release
	@install -D $(call fn_target_path,x86_64-unknown-linux-musl/release) \
		$(call fn_output_path,x86-64-v4-musl)

# x86-64 GLIBC

.PHONY: build-x86-64-gnu
build-x86-64-gnu: init $(call fn_output_path,x86-64-gnu)

$(call fn_output_path,x86-64-gnu) &: $(PROJECT_SRCS)
	@echo "Building for x86_64-unknown-linux-gnu"
	@cargo build -p nut_webgui --target=x86_64-unknown-linux-gnu --release
	@install -D $(call fn_target_path,x86_64-unknown-linux-gnu/release) \
		$(call fn_output_path,x86-64-gnu)

# x86-64-v3 GLIBC

.PHONY: build-x86-64-v3-gnu
build-x86-64-v3-gnu: init $(call fn_output_path,x86-64-v3-gnu)

$(call fn_output_path,x86-64-v3-gnu) &: $(PROJECT_SRCS)
	@echo "Building for x86_64-v3-unknown-linux-gnu"
	@export RUSTFLAGS="-Ctarget-cpu=x86-64-v3" && \
		cargo build -p nut_webgui --target=x86_64-unknown-linux-gnu --release
	@install -D $(call fn_target_path,x86_64-unknown-linux-gnu/release) \
		$(call fn_output_path,x86-64-v3-gnu)

# x86-64-v4 GLIBC

.PHONY: build-x86-64-v4-gnu
build-x86-64-v4-gnu: init $(call fn_output_path,x86-64-v4-gnu)

$(call fn_output_path,x86-64-v4-gnu) &: $(PROJECT_SRCS)
	@echo "Building for x86_64-v4-unknown-linux-gnu"
	@export RUSTFLAGS="-Ctarget-cpu=x86-64-v4" && \
		cargo build -p nut_webgui --target=x86_64-unknown-linux-gnu --release
	@install -D $(call fn_target_path,x86_64-unknown-linux-gnu/release) \
		$(call fn_output_path,x86-64-v4-gnu)

# ARM64/v8 MUSL

.PHONY: build-aarch64-musl
build-aarch64-musl: init $(call fn_output_path,aarch64-musl)

$(call fn_output_path,aarch64-musl) &: $(PROJECT_SRCS)
	@echo "Building for aarch64-unknown-linux-musl"
	@export RUSTFLAGS="-Clink-self-contained=yes -Clinker=rust-lld" && \
		cargo build -p nut_webgui --target=aarch64-unknown-linux-musl --release
	@install -D $(call fn_target_path,aarch64-unknown-linux-musl/release) \
		$(call fn_output_path,aarch64-musl)

# ARM64/v8 GLIBC

.PHONY: build-aarch64-gnu
build-aarch64-gnu: init $(call fn_output_path,aarch64-gnu)

$(call fn_output_path,aarch64-gnu) &: $(PROJECT_SRCS)
	@echo "Building for aarch64-unknown-linux-gnu"
	@export RUSTFLAGS="-Clinker=aarch64-linux-gnu-gcc" && \
		cargo build -p nut_webgui --target=aarch64-unknown-linux-gnu --release
	@install -D $(call fn_target_path,aarch64-unknown-linux-gnu/release) \
		$(call fn_output_path,aarch64-gnu)

# ARMv7 MUSL

.PHONY: build-armv7-musleabi
build-armv7-musleabi: init $(call fn_output_path,armv7-musleabi)

$(call fn_output_path,armv7-musleabi) &: $(PROJECT_SRCS)
	@echo "Building for armv7-unknown-linux-musleabi"
	@export RUSTFLAGS="-Clink-self-contained=yes -Clinker=rust-lld" && \
		export CC="clang" && \
		cargo build -p nut_webgui --target=armv7-unknown-linux-musleabi --release
	@install -D $(call fn_target_path,armv7-unknown-linux-musleabi/release) \
		$(call fn_output_path,armv7-musleabi)

# ARMv6 MUSL

.PHONY: build-armv6-musleabi
build-armv6-musleabi: init $(call fn_output_path,armv6-musleabi)

$(call fn_output_path,armv6-musleabi) &: $(PROJECT_SRCS)
	@echo "Building for arm-unknown-linux-musleabi"
	@export RUSTFLAGS="-Clink-self-contained=yes -Clinker=rust-lld" && \
		export CC="clang" && \
		export CFLAGS="--target=armv6-unknown-linux-musleabi" && \
		cargo build -p nut_webgui --target=arm-unknown-linux-musleabi --release
	@install -D $(call fn_target_path,arm-unknown-linux-musleabi/release) \
		$(call fn_output_path,armv6-musleabi)

# RISC-V64 GLIBC

.PHONY: build-riscv64gc-gnu
build-riscv64gc-gnu: init $(call fn_output_path,riscv64gc-gnu)

$(call fn_output_path,riscv64gc-gnu) &: $(PROJECT_SRCS)
	@echo "Building for riscv64gc-unknown-linux-gnu"
	@export RUSTFLAGS="-Clinker=riscv64-linux-gnu-gcc -Ctarget-feature=+crt-static" && \
		export CC="riscv64-linux-gnu-gcc" && \
		cargo build -p nut_webgui --target=riscv64gc-unknown-linux-gnu --release
	@install -D $(call fn_target_path,riscv64gc-unknown-linux-gnu/release) \
		$(call fn_output_path,riscv64gc-gnu)

.PHONY: build-all
build-all: $(addprefix build-,$(BINARY_TARGETS))

.PHONY: pack
pack: build-all
	@install -d $(DIST_DIR)
	@for target in $(BINARY_TARGETS); do \
		if [ -f "$(BIN_DIR)/$${target}/$(PROJECT_NAME)" ]; then \
			OUTPUT_TARGZ="$(DIST_DIR)/$(PROJECT_NAME)_$(PROJECT_VER)_$${target}.tar.gz"; \
			tar -czf "$${OUTPUT_TARGZ}" -C "$(BIN_DIR)/" "$${target}"; \
			echo "Packed $${target}.tar.gz"; \
			sha256sum "$${OUTPUT_TARGZ}" > "$${OUTPUT_TARGZ}.sha256"; \
			echo "Generated $${target}.tar.gz.sha256"; \
		fi; \
	done;

.PHONY: gen-dockerfiles
gen-dockerfiles:
	@install -d "$(BIN_DIR)/dockerfiles"
	@for entry in $$(jq -rc '.oci.images[]' "$(BUILD_CONFIG)"); do \
			export PLATFORM="$$(echo $$entry | jq -r '.platform')"; \
			export TARGET="$$(echo $$entry | jq -r '.target')"; \
			export BASE_CONTAINER_IMAGE="$$(echo $$entry | jq -r '.base_image')"; \
			export EXE_DIR="$(BIN_DIR)/$$TARGET"; \
			echo "Creating $${TARGET}.dockerfile"; \
			cat "$(DOCKER_TEMPLATE)" | envsubst > "$(BIN_DIR)/dockerfiles/$${TARGET}.Dockerfile"; \
		done;
	@echo "Creating annotation.json"
	@REVISION="$$(git rev-parse --verify HEAD)"; \
		cargo metadata \
			--no-deps \
			--frozen \
			--format-version 1 \
			--manifest-path "./nut_webgui/Cargo.toml" \
		| jq -r \
			--arg revision "$$REVISION" \
			'.packages[0] | { title:.name, version:.version, url:.homepage, licenses:.license, documentation:.documentation, source:.repository, description:.description, authors:(.authors | join(";")), revision: $$revision}' \
		> "$(BIN_DIR)/dockerfiles/annotations.json";

.PHONY: test
test: init
	@cargo test --all-features

.PHONY: check
check: init
	@cargo check --all-features

.PHONY: clean
clean:
	@echo "Cleaning artifacts"
	@cargo clean
	@if [ -d "$(BIN_DIR)" ]; then rm -r "$(BIN_DIR)"; fi;
	@if [ -d "$(NODE_MODULES_DIR)" ]; then rm -r "$(NODE_MODULES_DIR)"; fi;
	@echo "Clean completed"

.PHONY: watch
watch: init
	@if [ $$(which bacon 2>/dev/null) ]; then \
		bacon -j "serve"; \
	else \
		echo "Cannot find bacon. Watch function relies on bacon utility. See: https://github.com/Canop/bacon"; \
	fi
