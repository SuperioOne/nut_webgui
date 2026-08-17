ifdef ENABLE_RISCV_GNU
.PHONY: riscv64-gnu
riscv64-gnu:
	@echo "Building: riscv-gnu"
	@export RUSTFLAGS="-Clinker=riscv64-linux-gnu-gcc" && \
		export CC="riscv64-linux-gnu-gcc" && \
		cargo build -p nut_webgui --target=riscv64gc-unknown-linux-gnu --release
	@install -D "./target/riscv64gc-unknown-linux-gnu/release/nut_webgui" "$(ARTIFACT_DIR)/riscv64-gnu/nut_webgui"
endif

ifdef ENABLE_RISCV_MUSL
.PHONY: riscv64-musl
riscv64-musl:
	@echo "Building: riscv-musl"
	@export RUSTFLAGS="-Clink-self-contained=yes -Clinker=riscv64-linux-musl-gcc -Ctarget-feature=+crt-static" && \
		export CC="riscv64-linux-musl-gcc" && \
		cargo build -p nut_webgui --target=riscv64gc-unknown-linux-musl --release
	@install -D "./target/riscv64gc-unknown-linux-musl/release/nut_webgui" "$(ARTIFACT_DIR)/riscv64-musl/nut_webgui"
endif
