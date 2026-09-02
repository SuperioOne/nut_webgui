DOCS_DIR     := $(BIN_DIR)/docs
DOCS_SRC_DIR := ./docs

.PHONY: watch-docs
watch-docs:
	@mdbook serve "$(DOCS_SRC_DIR)"

.PHONY: build-docs
build-docs:
	@mdbook build -d "$(DOCS_DIR)/book" "$(DOCS_SRC_DIR)"
	@TARGET="$$(realpath -e "$(DOCS_DIR)")/book.zip"; \
		cd "$(DOCS_DIR)/book" && zip -r "$$TARGET" .
