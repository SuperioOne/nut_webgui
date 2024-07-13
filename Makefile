PROJECT_NAME  := nut_webgui
BIN_DIR       := ./bin
STATIC_DIR    := ./client/dist
PROJECT_SRCS  := $(shell find server/src -type f -iname *.rs) \
                 $(shell find server/src -type f -iname *.html) \
                 ./server/Cargo.toml \
                 ./server/Cargo.lock
STATIC_SRCS   := client/package.json \
                 client/pnpm-lock.yaml \
                 client/tailwind.config.js \
                 client/src/style.css \
                 $(shell find client/src -type f -iname *.js) \
                 $(shell find client/static -type f) \
                 $(shell find server/src -type f -iname *.html)
STATIC_OBJS   := $(addprefix $(BIN_DIR)/static/,index.js style.css icon.svg)
DIST_DIR       = $(BIN_DIR)/dist/
PACK_TARGETS   = x86-64-musl \
                 x86-64-v3-musl \
                 x86-64-v4-musl \
                 aarch64-musl \
                 aarch64-gnu \
                 armv6-musleabi \
                 armv7-musleabi \
                 riscv64gc-gnu

fn_output_path = $(BIN_DIR)/$(1)/$(PROJECT_NAME)
fn_target_path = server/target/$(1)/$(PROJECT_NAME)

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
# build-client         : Generates front-end static files. Requires node and pnpm.
# build-all            : Cross compiles everything. Make sure host system has all
#                        the necessary libs and tools for arm, x86-64 and riscv.
# pack                 : tar.gz all compiled targets under the bin directory.
# update-deps          : Updates both server and client dependencies.
# test                 : Calls available test suites.
# clean                : Clears all build directories.

.PHONY: build
build: $(call fn_output_path,release) build-client

$(call fn_output_path,release) &: $(PROJECT_SRCS)
	@echo "Building binaries for the current system's architecture."
	@cd ./server && cargo build --release
	@install -D $(call fn_target_path,release) $(call fn_output_path,release)

# x86-64 with different micro-architecture levels

.PHONY: build-x86-64-musl
build-x86-64-musl: $(call fn_output_path,x86-64-musl) build-client

$(call fn_output_path,x86-64-musl) &: $(PROJECT_SRCS)
	@echo "Building for x86_64-unknown-linux-musl"
	@cd ./server && \
		export RUSTFLAGS="-Clink-self-contained=yes -Clinker=rust-lld" && \
		cargo build --target=x86_64-unknown-linux-musl --release
	@install -D $(call fn_target_path,x86_64-unknown-linux-musl/release) \
		$(call fn_output_path,x86-64-musl)

.PHONY: build-x86-64-v3-musl
build-x86-64-v3-musl: $(call fn_output_path,x86-64-v3-musl) build-client

$(call fn_output_path,x86-64-v3-musl) &: $(PROJECT_SRCS)
	@echo "Building for x86_64-v3-unknown-linux-musl"
	@cd ./server && \
		export RUSTFLAGS="-Clink-self-contained=yes -Ctarget-cpu=x86-64-v3 -Clinker=rust-lld" && \
		cargo build --target=x86_64-unknown-linux-musl --release
	@install -D $(call fn_target_path,x86_64-unknown-linux-musl/release) \
		$(call fn_output_path,x86-64-v3-musl)

.PHONY: build-x86-64-v4-musl
build-x86-64-v4-musl: $(call fn_output_path,x86-64-v4-musl) build-client

$(call fn_output_path,x86-64-v4-musl) &: $(PROJECT_SRCS)
	@echo "Building for x86_64-v4-unknown-linux-musl"
	@cd ./server && \
		export RUSTFLAGS="-Clink-self-contained=yes -Ctarget-cpu=x86-64-v4 -Clinker=rust-lld" && \
		cargo build --target=x86_64-unknown-linux-musl --release
	@install -D $(call fn_target_path,x86_64-unknown-linux-musl/release) \
		$(call fn_output_path,x86-64-v4-musl)

# ARMv8

.PHONY: build-aarch64-musl
build-aarch64-musl: $(call fn_output_path,aarch64-musl) build-client

$(call fn_output_path,aarch64-musl) &: $(PROJECT_SRCS)
	@echo "Building for aarch64-unknown-linux-musl"
	@cd ./server && \
		export RUSTFLAGS="-Clink-self-contained=yes -Clinker=rust-lld" && \
		cargo build --target=aarch64-unknown-linux-musl --release
	@install -D $(call fn_target_path,aarch64-unknown-linux-musl/release) \
		$(call fn_output_path,aarch64-musl)

.PHONY: build-aarch64-gnu
build-aarch64-gnu: $(call fn_output_path,aarch64-gnu) build-client

$(call fn_output_path,aarch64-gnu) &: $(PROJECT_SRCS)
	@echo "Building for aarch64-unknown-linux-gnu"
	@cd ./server && \
		export RUSTFLAGS="-Clinker=aarch64-linux-gnu-gcc" && \
		cargo build --target=aarch64-unknown-linux-gnu --release
	@install -D $(call fn_target_path,aarch64-unknown-linux-gnu/release) \
		$(call fn_output_path,aarch64-gnu)

# ARMv7

.PHONY: build-armv7-musleabi
build-armv7-musleabi: $(call fn_output_path,armv7-musleabi) build-client

$(call fn_output_path,armv7-musleabi) &: $(PROJECT_SRCS)
	@echo "Building for armv7-unknown-linux-musleabi"
	@cd ./server && \
		export RUSTFLAGS="-Clink-self-contained=yes -Clinker=rust-lld" && \
		cargo build --target=armv7-unknown-linux-musleabi --release
	@install -D $(call fn_target_path,armv7-unknown-linux-musleabi/release) \
		$(call fn_output_path,armv7-musleabi)

# ARMv6

.PHONY: build-armv6-musleabi
build-armv6-musleabi: $(call fn_output_path,armv6-musleabi) build-client

$(call fn_output_path,armv6-musleabi) &: $(PROJECT_SRCS)
	@echo "Building for arm-unknown-linux-musleabi"
	@cd ./server && \
		export RUSTFLAGS="-Clink-self-contained=yes -Clinker=rust-lld" && \
		cargo build --target=arm-unknown-linux-musleabi --release
	@install -D $(call fn_target_path,arm-unknown-linux-musleabi/release) \
		$(call fn_output_path,armv6-musleabi)

# RISC-V64

.PHONY: build-riscv64gc-gnu
build-riscv64gc-gnu: $(call fn_output_path,riscv64gc-gnu) build-client

$(call fn_output_path,riscv64gc-gnu) &: $(PROJECT_SRCS)
	@echo "Building for riscv64gc-unknown-linux-gnu"
	@cd ./server && \
		export RUSTFLAGS="-Clinker=riscv64-linux-gnu-gcc" && \
		cargo build --target=riscv64gc-unknown-linux-gnu --release
	@install -D $(call fn_target_path,riscv64gc-unknown-linux-gnu/release) \
		$(call fn_output_path,riscv64gc-gnu)

.PHONY: build-client
build-client: $(STATIC_OBJS)

$(STATIC_OBJS) &: $(STATIC_SRCS)
	@pnpm install -C ./client/
	@pnpm run -C ./client/ build --outdir=../$(BIN_DIR)/static --minify

.PHONY: build-all
build-all: $(addprefix build-,$(PACK_TARGETS))

.PHONY: pack
pack: $(STATIC_OBJS)
	@install -d $(DIST_DIR)
	@for target in $(PACK_TARGETS); do \
		if [ -f "$(BIN_DIR)/$${target}/$(PROJECT_NAME)" ]; then \
			echo "Packing $${target}.tar.gz"; \
			cp -r "$(BIN_DIR)/static" "$(BIN_DIR)/$${target}"; \
			tar -czf "$(DIST_DIR)/$${target}.tar.gz" -C "$(BIN_DIR)/" "$${target}"; \
		fi; \
	done;

.PHONY: update-deps
update-deps:
	@echo "Updating client dependencies."
	@pnpm outdated -C ./client/
	@pnpm update -C ./client/
	@echo "Updating server dependencies."
	@cd ./server/ && cargo update

.PHONY: test
test:
	@cd ./server && cargo test

.PHONY: clean
clean:
	@echo "Cleaning artifacts"
	@cd server && cargo clean
	@test -d $(BIN_DIR) && rm -r $(BIN_DIR)
	@test -d $(STATIC_DIR) && rm -r $(STATIC_DIR)
	@echo "Clean completed"
