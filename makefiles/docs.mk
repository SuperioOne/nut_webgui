DOCS_DIR := ./docs

.PHONY: serve-book
serve-book:
	mdbook serve "$(DOCS_DIR)"
