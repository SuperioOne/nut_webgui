ifdef ENABLE_RISCV_GNU
$(ARTIFACT_DIR)/riscv64gc-gnu/nut_webgui: $(PROJECT_SRC)
	@export RUSTFLAGS="-Clinker=riscv64-linux-gnu-gcc" && \
		export CC="riscv64-linux-gnu-gcc" && \
		cargo build -p nut_webgui --target=riscv64gc-unknown-linux-gnu --release
	@install -D "./target/riscv64gc-unknown-linux-gnu/release/nut_webgui" "$(ARTIFACT_DIR)/riscv64gc-gnu/nut_webgui"

.PHONY: riscv64gc-gnu
riscv64gc-gnu: $(ARTIFACT_DIR)/riscv64gc-gnu/nut_webgui
endif

ifdef ENABLE_RISCV_MUSL
$(ARTIFACT_DIR)/riscv64gc-musl/nut_webgui: $(PROJECT_SRC)
	@export RUSTFLAGS="-Clink-self-contained=yes -Clinker=riscv64-linux-musl-gcc -Ctarget-feature=+crt-static" && \
		export CC="riscv64-linux-musl-gcc" && \
		cargo build -p nut_webgui --target=riscv64gc-unknown-linux-musl --release
	@install -D "./target/riscv64gc-unknown-linux-musl/release/nut_webgui" "$(ARTIFACT_DIR)/riscv64gc-musl/nut_webgui"

.PHONY: riscv64gc-musl
riscv64gc-musl: $(ARTIFACT_DIR)/riscv64gc-musl/nut_webgui
endif
