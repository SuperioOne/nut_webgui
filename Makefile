PROJECT_NAME     := $(shell cargo read-manifest --manifest-path ./nut_webgui/Cargo.toml | jq -r ".name")
PROJECT_VER      := $(shell cargo read-manifest --manifest-path ./nut_webgui/Cargo.toml | jq -r ".version")
BIN_DIR          := ./bin
NODE_MODULES_DIR := ./nut_webgui_client/node_modules
PROJECT_SRCS     := $(shell git ls-tree -r HEAD --name-only | grep "nut_webgui")
DIST_DIR          = $(BIN_DIR)/dist
DOCKER_TEMPLATE   = ./Dockerfile.template
PACK_TARGETS      = x86-64-musl \
					x86-64-v3-musl \
					x86-64-v4-musl \
					aarch64-musl \
					aarch64-gnu \
					armv6-musleabi \
					armv7-musleabi \
					riscv64gc-gnu

fn_output_path    = $(BIN_DIR)/$(1)/$(PROJECT_NAME)
fn_target_path    = nut_webgui/target/$(1)/$(PROJECT_NAME)

# RECIPIES:
# ==============================================================================
# build                : Generates server binary for the current system's CPU 
#                        architecture and OS.
# build-x86-64-musl    : linux/amd64, self contained with musl.
# build-x86-64-v3-musl : linux/amd64/v3, self contained with musl.
# build-x86-64-v4-musl : linux/amd64/v4, self contained with musl.
# build-aarch64-musl   : linux/arm64/v8, self contained with musl.
# build-aarch64-gnu    : linux/arm64/v8, glibc.
# build-armv6-musleabi : linux/arm/v6, self contained with musl, soft-floats.
# build-armv7-musleabi : linux/arm/v7, self contained with musl, soft-floats.
# build-riscv64gc-gnu  : linux/riscv64, glibc.
# build-all            : Cross compiles everything. Make sure host system has all
#                        the necessary libs and tools for arm, x86-64 and riscv.
# generate-dockerfiles : Generates dockerfiles for all supported architectures.
# pack                 : tar.gz all compiled targets under the bin directory.
# test                 : Calls available test suites.
# clean                : Clears all build directories.

.PHONY: init_project
init_project:
	@pnpm install -C ./nut_webgui_client

.PHONY: build
build: init_project $(call fn_output_path,release)

$(call fn_output_path,release) &: $(PROJECT_SRCS)
	@echo "Building binaries for the current system's architecture."
	@cd nut_webgui && cargo build --release
	@install -D $(call fn_target_path,release) $(call fn_output_path,release)

# x86-64 with different micro-architecture levels

.PHONY: build-x86-64-musl
build-x86-64-musl: init_project $(call fn_output_path,x86-64-musl) init_project

$(call fn_output_path,x86-64-musl) &: $(PROJECT_SRCS)
	@echo "Building for x86_64-unknown-linux-musl"
	@cd nut_webgui && \
		export RUSTFLAGS="-Clink-self-contained=yes -Clinker=rust-lld" && \
		cargo build --target=x86_64-unknown-linux-musl --release
	@install -D $(call fn_target_path,x86_64-unknown-linux-musl/release) \
		$(call fn_output_path,x86-64-musl)

.PHONY: build-x86-64-v3-musl
build-x86-64-v3-musl: init_project $(call fn_output_path,x86-64-v3-musl)

$(call fn_output_path,x86-64-v3-musl) &: $(PROJECT_SRCS)
	@echo "Building for x86_64-v3-unknown-linux-musl"
	@cd nut_webgui && \
		export RUSTFLAGS="-Clink-self-contained=yes -Ctarget-cpu=x86-64-v3 -Clinker=rust-lld" && \
		cargo build --target=x86_64-unknown-linux-musl --release
	@install -D $(call fn_target_path,x86_64-unknown-linux-musl/release) \
		$(call fn_output_path,x86-64-v3-musl)

.PHONY: build-x86-64-v4-musl
build-x86-64-v4-musl: init_project $(call fn_output_path,x86-64-v4-musl)

$(call fn_output_path,x86-64-v4-musl) &: $(PROJECT_SRCS)
	@echo "Building for x86_64-v4-unknown-linux-musl"
	@cd nut_webgui && \
		export RUSTFLAGS="-Clink-self-contained=yes -Ctarget-cpu=x86-64-v4 -Clinker=rust-lld" && \
		cargo build --target=x86_64-unknown-linux-musl --release
	@install -D $(call fn_target_path,x86_64-unknown-linux-musl/release) \
		$(call fn_output_path,x86-64-v4-musl)

# ARM64v8

.PHONY: build-aarch64-musl
build-aarch64-musl: init_project $(call fn_output_path,aarch64-musl)

$(call fn_output_path,aarch64-musl) &: $(PROJECT_SRCS)
	@echo "Building for aarch64-unknown-linux-musl"
	@cd nut_webgui && \
		export RUSTFLAGS="-Clink-self-contained=yes -Clinker=rust-lld" && \
		cargo build --target=aarch64-unknown-linux-musl --release
	@install -D $(call fn_target_path,aarch64-unknown-linux-musl/release) \
		$(call fn_output_path,aarch64-musl)

.PHONY: build-aarch64-gnu
build-aarch64-gnu: init_project $(call fn_output_path,aarch64-gnu)

$(call fn_output_path,aarch64-gnu) &: $(PROJECT_SRCS)
	@echo "Building for aarch64-unknown-linux-gnu"
	@cd nut_webgui && \
		export RUSTFLAGS="-Clinker=aarch64-linux-gnu-gcc" && \
		cargo build --target=aarch64-unknown-linux-gnu --release
	@install -D $(call fn_target_path,aarch64-unknown-linux-gnu/release) \
		$(call fn_output_path,aarch64-gnu)

# ARMv7

.PHONY: build-armv7-musleabi
build-armv7-musleabi: init_project $(call fn_output_path,armv7-musleabi)

$(call fn_output_path,armv7-musleabi) &: $(PROJECT_SRCS)
	@echo "Building for armv7-unknown-linux-musleabi"
	@cd nut_webgui && \
		export RUSTFLAGS="-Clink-self-contained=yes -Clinker=rust-lld" && \
		cargo build --target=armv7-unknown-linux-musleabi --release
	@install -D $(call fn_target_path,armv7-unknown-linux-musleabi/release) \
		$(call fn_output_path,armv7-musleabi)

# ARMv6

.PHONY: build-armv6-musleabi
build-armv6-musleabi: init_project $(call fn_output_path,armv6-musleabi)

$(call fn_output_path,armv6-musleabi) &: $(PROJECT_SRCS)
	@echo "Building for arm-unknown-linux-musleabi"
	@cd nut_webgui && \
		export RUSTFLAGS="-Clink-self-contained=yes -Clinker=rust-lld" && \
		cargo build --target=arm-unknown-linux-musleabi --release
	@install -D $(call fn_target_path,arm-unknown-linux-musleabi/release) \
		$(call fn_output_path,armv6-musleabi)

# RISC-V64

.PHONY: build-riscv64gc-gnu
build-riscv64gc-gnu: init_project $(call fn_output_path,riscv64gc-gnu)

$(call fn_output_path,riscv64gc-gnu) &: $(PROJECT_SRCS)
	@echo "Building for riscv64gc-unknown-linux-gnu"
	@cd nut_webgui && \
		export RUSTFLAGS="-Clinker=riscv64-linux-gnu-gcc -Ctarget-feature=+crt-static" && \
		cargo build --target=riscv64gc-unknown-linux-gnu --release
	@install -D $(call fn_target_path,riscv64gc-unknown-linux-gnu/release) \
		$(call fn_output_path,riscv64gc-gnu)


.PHONY: build-all
build-all: $(addprefix build-,$(PACK_TARGETS))

.PHONY: pack
pack:
	@install -d $(DIST_DIR)
	@for target in $(PACK_TARGETS); do \
		if [ -f "$(BIN_DIR)/$${target}/$(PROJECT_NAME)" ]; then \
			OUTPUT_TARGZ="$(DIST_DIR)/$(PROJECT_NAME)_$(PROJECT_VER)_$${target}.tar.gz"; \
			tar -czf "$${OUTPUT_TARGZ}" -C "$(BIN_DIR)/" "$${target}"; \
			echo "Packed $${target}.tar.gz"; \
			sha256sum "$${OUTPUT_TARGZ}" > "$${OUTPUT_TARGZ}.sha256"; \
			echo "Generated $${target}.tar.gz.sha256"; \
		fi; \
	done;

.PHONY: test
test:
	@cd nut_webgui && cargo test
	@cd nut_webgui_upsmc && cargo test --all-features

.PHONY: check
check:
	@cd nut_webgui && cargo check
	@cd nut_webgui_upsmc && cargo check --all-features

.PHONY: generate-dockerfiles
generate-dockerfiles: 
	@install -d "$(BIN_DIR)/containers"
	@echo "amd64.Dockerfile"
	@sed -e 's/{BIN_DIR}/x86-64-musl/g' \
		-e 's/{PLATFORM}/linux\/amd64/g' \
		-e 's/{BUSYBOX_LABEL}/stable-musl/g' \
		"$(DOCKER_TEMPLATE)" > "$(BIN_DIR)/containers/amd64.Dockerfile"
	@echo "amd64-v3.Dockerfile"
	@sed -e 's/{BIN_DIR}/x86-64-v3-musl/g' \
		-e 's/{PLATFORM}/linux\/amd64/g' \
		-e 's/{BUSYBOX_LABEL}/stable-musl/g' \
		"$(DOCKER_TEMPLATE)" > "$(BIN_DIR)/containers/amd64-v3.Dockerfile"
	@echo "amd64-v4.Dockerfile"
	@sed -e 's/{BIN_DIR}/x86-64-v4-musl/g' \
		-e 's/{PLATFORM}/linux\/amd64/g' \
		-e 's/{BUSYBOX_LABEL}/stable-musl/g' \
		"$(DOCKER_TEMPLATE)" > "$(BIN_DIR)/containers/amd64-v4.Dockerfile"
	@echo "arm64.Dockerfile"
	@sed -e 's/{BIN_DIR}/aarch64-musl/g' \
		-e 's/{PLATFORM}/linux\/arm64\/v8/g' \
		-e 's/{BUSYBOX_LABEL}/stable-musl/g' \
		"$(DOCKER_TEMPLATE)" > "$(BIN_DIR)/containers/arm64.Dockerfile"
	@echo "armv7.Dockerfile"
	@sed -e 's/{BIN_DIR}/armv7-musleabi/g' \
		-e 's/{PLATFORM}/linux\/arm\/v7/g' \
		-e 's/{BUSYBOX_LABEL}/stable-musl/g' \
		"$(DOCKER_TEMPLATE)" > "$(BIN_DIR)/containers/armv7.Dockerfile"
	@echo "armv6.Dockerfile"
	@sed -e 's/{BIN_DIR}/armv6-musleabi/g' \
		-e 's/{PLATFORM}/linux\/arm\/v6/g' \
		-e 's/{BUSYBOX_LABEL}/stable-musl/g' \
		"$(DOCKER_TEMPLATE)" > "$(BIN_DIR)/containers/armv6.Dockerfile"
	@echo "riscv64.Dockerfile"
	@sed -e 's/{BIN_DIR}/riscv64gc-gnu/g' \
		-e 's/{PLATFORM}/linux\/riscv64/g' \
		-e 's/{BUSYBOX_LABEL}/stable-musl/g' \
		"$(DOCKER_TEMPLATE)" > "$(BIN_DIR)/containers/riscv64.Dockerfile"
	@echo "Generating annotation.conf"
	@CARGO_MANIFEST=$$(cargo read-manifest --manifest-path ./nut_webgui/Cargo.toml); \
	echo "VERSION=\"$$(echo -n "$${CARGO_MANIFEST}" | jq -r ".version")\"" > "$(BIN_DIR)/containers/annotation.conf"; \
	echo "HOME_URL=\"$$(echo -n "$${CARGO_MANIFEST}" | jq -r ".homepage")\"" >> "$(BIN_DIR)/containers/annotation.conf"; \
	echo "NAME=\"$$(echo -n "$${CARGO_MANIFEST}" | jq -r ".name")\"" >> "$(BIN_DIR)/containers/annotation.conf"; \
	echo "LICENSES=\"$$(echo -n "$${CARGO_MANIFEST}" | jq -r ".license")\"" >> "$(BIN_DIR)/containers/annotation.conf"; \
	echo "AUTHORS=\"$$(echo -n "$${CARGO_MANIFEST}" | jq -r '.authors | join(" ")')\"" >> "$(BIN_DIR)/containers/annotation.conf"; \
	echo "DOCUMENTATION=\"$$(echo -n "$${CARGO_MANIFEST}" | jq -r ".documentation")\"" >> "$(BIN_DIR)/containers/annotation.conf"; \
	echo "SOURCE=\"$$(echo -n "$${CARGO_MANIFEST}" | jq -r ".repository")\"" >> "$(BIN_DIR)/containers/annotation.conf"; \
	echo "DESCRIPTION=\"$$(echo -n "$${CARGO_MANIFEST}" | jq -r ".description")\"" >> "$(BIN_DIR)/containers/annotation.conf"; \
	echo "REVISION=\"$$(git rev-parse --verify HEAD)\"" >> "$(BIN_DIR)/containers/annotation.conf";
	
.PHONY: clean
clean:
	@echo "Cleaning artifacts"
	@cd nut_webgui && cargo clean
	@cd nut_webgui_upsmc && cargo clean
	@cd nut_webgui_client && cargo clean
	@if [ -d "$(BIN_DIR)" ]; then rm -r "$(BIN_DIR)"; fi;
	@if [ -d "$(NODE_MODULES_DIR)" ]; then rm -r "$(NODE_MODULES_DIR)"; fi;
	@echo "Clean completed"

