ifdef ENABLE_AARCH64_MUSL
$(ARTIFACT_DIR)/aarch64-musl/nut_webgui: $(PROJECT_SRC)
	@export RUSTFLAGS="-Clink-self-contained=yes -Clinker=rust-lld" && \
		cargo build -p nut_webgui --target=aarch64-unknown-linux-musl --release
	@install -D "./target/aarch64-unknown-linux-musl/release/nut_webgui" "$(ARTIFACT_DIR)/aarch64-musl/nut_webgui"

.PHONY: aarch64-musl
aarch64-musl: $(ARTIFACT_DIR)/aarch64-musl/nut_webgui
endif

ifdef ENABLE_AARCH64_GNU
$(ARTIFACT_DIR)/aarch64-gnu/nut_webgui: $(PROJECT_SRC)
	@export RUSTFLAGS="-Clinker=aarch64-linux-gnu-gcc" && \
		cargo build -p nut_webgui --target=aarch64-unknown-linux-gnu --release
	@install -D "./target/aarch64-unknown-linux-gnu/release/nut_webgui" "$(ARTIFACT_DIR)/aarch64-gnu/nut_webgui"

.PHONY: aarch64-gnu
aarch64-gnu: $(ARTIFACT_DIR)/aarch64-gnu/nut_webgui
endif

ifdef ENABLE_ARMV7_MUSLEABI
$(ARTIFACT_DIR)/armv7-musleabi/nut_webgui: $(PROJECT_SRC)
	@export RUSTFLAGS="-Clink-self-contained=yes -Clinker=rust-lld" && \
		export CC="clang" && \
		cargo build -p nut_webgui --target=armv7-unknown-linux-musleabi --release
	@install -D "./target/armv7-unknown-linux-musleabi/release/nut_webgui" "$(ARTIFACT_DIR)/armv7-musleabi/nut_webgui"

.PHONY: armv7-musleabi
armv7-musleabi: $(ARTIFACT_DIR)/armv7-musleabi/nut_webgui
endif

ifdef ENABLE_ARMV6_MUSLEABI
$(ARTIFACT_DIR)/armv6-musleabi/nut_webgui: $(PROJECT_SRC)
	@export RUSTFLAGS="-Clink-self-contained=yes -Clinker=rust-lld" && \
		export CC="clang" && \
		export CFLAGS="--target=armv6-unknown-linux-musleabi" && \
		cargo build -p nut_webgui --target=arm-unknown-linux-musleabi --release
	@install -D "./target/arm-unknown-linux-musleabi/release/nut_webgui" "$(ARTIFACT_DIR)/armv6-musleabi/nut_webgui"

.PHONY: armv6-musleabi
armv6-musleabi: $(ARTIFACT_DIR)/armv6-musleabi/nut_webgui
endif

