ifdef ENABLE_X86_64_MUSL
$(ARTIFACT_DIR)/x86-64-musl/nut_webgui: $(PROJECT_SRC)
	@export RUSTFLAGS="-Clink-self-contained=yes -Clinker=rust-lld" && \
		cargo build -p nut_webgui --target=x86_64-unknown-linux-musl --release
	@install -D "./target/x86_64-unknown-linux-musl/release/nut_webgui" "$(ARTIFACT_DIR)/x86-64-musl/nut_webgui"

.PHONY: x86-64-musl
x86-64-musl: $(ARTIFACT_DIR)/x86-64-musl/nut_webgui
endif

ifdef ENABLE_X86_64_GNU
$(ARTIFACT_DIR)/x86-64-gnu/nut_webgui: $(PROJECT_SRC)
	@cargo build -p nut_webgui --target=x86_64-unknown-linux-gnu --release
	@install -D "./target/x86_64-unknown-linux-gnu/release/nut_webgui" "$(ARTIFACT_DIR)/x86-64-gnu/nut_webgui"

.PHONY: x86-64-gnu
x86-64-gnu: $(ARTIFACT_DIR)/x86-64-gnu/nut_webgui
endif

ifdef ENABLE_X86_64_V3_MUSL
$(ARTIFACT_DIR)/x86-64-v3-musl/nut_webgui: $(PROJECT_SRC)
	@export RUSTFLAGS="-Clink-self-contained=yes -Ctarget-cpu=x86-64-v3 -Clinker=rust-lld" && \
		cargo build -p nut_webgui --target=x86_64-unknown-linux-musl --target-dir target/x86-64-v3-musl --release
	@install -D "./target/x86-64-v3-musl/x86_64-unknown-linux-musl/release/nut_webgui" "$(ARTIFACT_DIR)/x86-64-v3-musl/nut_webgui"

.PHONY: x86-64-v3-musl
x86-64-v3-musl: $(ARTIFACT_DIR)/x86-64-v3-musl/nut_webgui
endif

ifdef ENABLE_X86_64_V3_GNU
$(ARTIFACT_DIR)/x86-64-v3-gnu/nut_webgui: $(PROJECT_SRC)
	@export RUSTFLAGS="-Ctarget-cpu=x86-64-v3" && \
		cargo build -p nut_webgui --target=x86_64-unknown-linux-gnu --target-dir target/x86_64-v3-gnu --release
	@install -D "./target/x86_64-v3-gnu/x86_64-unknown-linux-gnu/release/nut_webgui" "$(ARTIFACT_DIR)/x86-64-v3-gnu/nut_webgui"
	
.PHONY: x86-64-v3-gnu
x86-64-v3-gnu: $(ARTIFACT_DIR)/x86-64-v3-gnu/nut_webgui
endif

ifdef ENABLE_X86_64_V4_MUSL
$(ARTIFACT_DIR)/x86-64-v4-musl/nut_webgui: $(PROJECT_SRC)
	@export RUSTFLAGS="-Clink-self-contained=yes -Ctarget-cpu=x86-64-v4 -Clinker=rust-lld" && \
		cargo build -p nut_webgui --target=x86_64-unknown-linux-musl --target-dir target/x86_64-v4-musl --release
	@install -D "./target/x86_64-v4-musl/x86_64-unknown-linux-musl/release/nut_webgui" "$(ARTIFACT_DIR)/x86-64-v4-musl/nut_webgui"

.PHONY: x86-64-v4-musl
x86-64-v4-musl: $(ARTIFACT_DIR)/x86-64-v4-musl/nut_webgui
endif

ifdef ENABLE_X86_64_V4_GNU
$(ARTIFACT_DIR)/x86-64-v4-gnu/nut_webgui: $(PROJECT_SRC)
	@export RUSTFLAGS="-Ctarget-cpu=x86-64-v4" && \
		cargo build -p nut_webgui --target=x86_64-unknown-linux-gnu --target-dir target/x86_64-v4-gnu --release
	@install -D "./target/x86_64-v4-gnu/x86_64-unknown-linux-gnu/release/nut_webgui" "$(ARTIFACT_DIR)/x86-64-v4-gnu/nut_webgui"

.PHONY: x86-64-v4-gnu
x86-64-v4-gnu: $(ARTIFACT_DIR)/x86-64-v4-gnu/nut_webgui
endif
