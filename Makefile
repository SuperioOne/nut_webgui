BIN_DIR           := ./bin
ARTIFACT_DIR      := $(BIN_DIR)/artifact
NODE_MODULES_DIR  := ./nut_webgui_client/node_modules
PROJECT_SRC       := ./Cargo.toml \
											$(wildcard ./**/Cargo.toml) \
											$(wildcard ./nut_webgui/src/**/*.html) \
											$(wildcard ./nut_webgui/src/**/*.rs) \
											$(wildcard ./nut_webgui/src/*.rs) \
											$(wildcard ./nut_webgui_client/package.json) \
											$(wildcard ./nut_webgui_client/postcss.built.js) \
											$(wildcard ./nut_webgui_client/src/**/*.js) \
											$(wildcard ./nut_webgui_client/src/*.js) \
											$(wildcard ./nut_webgui_client/static/*) \
											$(wildcard ./nut_webgui_upsmc/src/**/*.rs) \
											$(wildcard ./nut_webgui_upsmc/src/*.rs)

-include .config.mk

ifdef TARGETS

include ./makefiles/x86-64.mk
include ./makefiles/arm.mk
include ./makefiles/riscv.mk
include ./makefiles/package_build.mk

ifdef GH_TARGETS
include ./makefiles/release_github.mk
endif

ifdef ENABLE_CONTAINER_IMAGE
include ./makefiles/image_build.mk

ifdef OCI_TARGETS
include ./makefiles/image_publish.mk
endif
endif

.PHONY: build-all
build-all: $(TARGETS)
endif

.PHONY: help
help:
	@echo "BASIC RECIPES"
	@echo "  build           : Build server binary for the current system's CPU architecture and OS."
	@echo "  build-native    : Build server binary specifically optimized for the current system's CPU."
	@echo "  clean           : Clear all build directories."
	@echo "  install         : Build nut_webgui and install it to /usr/bin/local (Requires permission)."
	@echo "  install-local   : Build nut_webgui and install it locally to $$HOME/.local/bin"
	@echo "  test            : Call test suites."
ifdef TARGETS
	@echo "CONFIG SPECIFIC RECIPES"
	@echo "  build-all       : Cross compile all configured targets."
	@echo "  package         : Pack all artifacts locally for distribution."
ifdef ENABLE_CONTAINER_IMAGE
	@echo "  build-images    : Build container images locally."
ifdef OCI_TARGETS
	@echo "  publish-images  : Build container images and publish to target container registries."
endif
endif
ifdef GH_TARGETS
	@echo "  release-github  : Pack all artifacts and create new draft release on target GitHub repos."
endif
ifdef FJ_TARGETS
	@echo "  release-forgejo : Pack all artifacts and create new draft release on target Forgejo/Gitea repos."
endif
endif

# Builds with default toolchain
.PHONY: build
build:
	@echo "Building binaries for the current system's architecture."
	@cargo build -p nut_webgui --release
	@install -D "./target/release/nut_webgui" "$(ARTIFACT_DIR)/release/nut_webgui"

# Builds with default toolchain and host CPU specific optimizations enabled.
.PHONY: build-native
build-native:
	@echo "Building binary for the current system's CPU."
	@export RUSTFLAGS="-Ctarget-cpu=native" && \
		cargo build -p nut_webgui --release
	@install -D "./target/release/nut_webgui" "$(ARTIFACT_DIR)/release/nut_webgui"

.PHONY: install
install: build-native
	@if [ -w "/usr/local/bin" ]; then \
			install "./target/release/nut_webgui" "/usr/local/bin/nut_webgui"; \
			echo "Install completed: /usr/local/bin/nut_webgui"; \
		else \
			echo "Current user cannot write into '/usr/local/bin'"; \
		exit 1; \
	 fi

.PHONY: install-local
install-local: build-native
	@install -D "./target/release/nut_webgui" "$$HOME/.local/bin/nut_webgui"
	@echo "Install completed: $$HOME/.local/bin/nut_webgui"

.PHONY: test
test:
	@cargo test --all-features

.PHONY: clean
clean:
	@echo "Cleaning artifacts"
	@cargo clean
	@if [ -d "$(BIN_DIR)" ]; then rm -r "$(BIN_DIR)"; fi;
	@if [ -d "$(NODE_MODULES_DIR)" ]; then rm -r "$(NODE_MODULES_DIR)"; fi;
	@echo "Cleanup completed"

.PHONY: watch
watch:
	@if [ $$(which bacon 2>/dev/null) ]; then \
		bacon -j "serve"; \
	else \
		echo "Cannot find bacon. Watch function relies on bacon utility. See: https://github.com/Canop/bacon"; \
	fi
