BIN_DIR           := ./bin
ARTIFACT_DIR      := "$(BIN_DIR)/artifact"
NODE_MODULES_DIR  := ./nut_webgui_client/node_modules

-include .config.mk

ifdef TARGETS

include ./makefiles/x86-64.mk
include ./makefiles/arm.mk
include ./makefiles/riscv.mk
include ./makefiles/package.mk

.PHONY: build-all
build-all: $(TARGETS)

endif

.PHONY: help
help:
	@echo "BASIC RECIPES"
	@echo "  build         : Build server binary for the current system's CPU architecture and OS."
	@echo "  build-native  : Build server binary specifically optimized for the current system's CPU."
	@echo "  clean         : Clear all build directories."
	@echo "  install       : Build nut_webgui and install it to /usr/bin/local (Requires permission)."
	@echo "  install-local : Build nut_webgui and install it locally to $$HOME/.local/bin"
	@echo "  test          : Call test suites."
ifdef TARGETS
	@echo ""
	@echo "CONFIG SPECIFIC RECIPES"
	@echo "  build-all     : Cross compile all configured targets."
	@echo "  package       : Pack all artifacts for release."
ifdef ENABLE_OCI_CONTAINER
	@echo "  build-images  : Build container images for the supported targets. (Only self-contained musl targets are supported)"
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

.PHONY: gen-pkgbuild
gen-pkgbuild: build-x86-64-gnu build-aarch64-gnu build-armv7-musleabi
	@echo "Creating PKGBUILD"
	@install -d $(DIST_DIR)
	@NUTWG_SHA256_x86_64="$$(cat "$(DIST_DIR)/$(PROJECT_NAME)_$(PROJECT_VER)_x86-64-gnu.tar.gz.sha256" | awk -F ' ' '{print $$1}')"; \
		NUTWG_SHA256_AARCH64="$$(cat "$(DIST_DIR)/$(PROJECT_NAME)_$(PROJECT_VER)_aarch64-gnu.tar.gz.sha256" | awk -F ' ' '{print $$1}')"; \
		NUTWG_SHA256_ARMV7="$$(cat "$(DIST_DIR)/$(PROJECT_NAME)_$(PROJECT_VER)_armv7-musleabi.tar.gz.sha256" | awk -F ' ' '{print $$1}')"; \
		cat "$(PKGBUILD_TEMPLATE)" \
		| sed -e "s/__PLACEHOLDER_NUTWG_VERSION/$(PROJECT_VER)/g" \
			-e "s/__PLACEHOLDER_NUTWG_SHA256_x86_64/$$NUTWG_SHA256_x86_64/g" \
			-e "s/__PLACEHOLDER_NUTWG_SHA256_AARCH64/$$NUTWG_SHA256_AARCH64/g" \
			-e "s/__PLACEHOLDER_NUTWG_SHA256_ARMV7/$$NUTWG_SHA256_ARMV7/g" > "$(DIST_DIR)/PKGBUILD"

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
