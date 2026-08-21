PACKAGE_DIR      := $(BIN_DIR)/packages
STAGING_DIR      := $(BIN_DIR)/staging
STAGING_TARGETS  := $(foreach TARGET,$(TARGETS),$(STAGING_DIR)/nut_webgui_$(VERSION)_$(TARGET))
PACKAGE_TARS     := $(foreach TARGET,$(TARGETS),$(PACKAGE_DIR)/nut_webgui_$(VERSION)_$(TARGET).tar.gz)
CHANGELOG_FILE   := $(STAGING_DIR)/CHANGELOG
MAN_PAGES_DIR    := $(STAGING_DIR)/man
INSTALL_SCRIPT   := $(PACKAGE_DIR)/install.sh
MANIFEST_FILE    := $(PACKAGE_DIR)/MANIFEST
PKGBUILD_ENABLED := $(or $(ENABLE_ARMV7_MUSLEABI),$(ENABLE_AARCH64_GNU),$(ENABLE_X86_64_GNU))
MAN_PAGE_SRCS    := ./dist/man/nut_webgui.1 \
										./dist/man/nut_webgui.7
PACKAGE_CONTENTS := $(CHANGELOG_FILE) \
										$(MAN_PAGES_DIR) \
										./LICENSE \
										./dist/config.toml \
										./dist/service \
										./dist/users.toml

ifdef PKGBUILD_ENABLED
PKGBUILD_SCRIPT  := $(PACKAGE_DIR)/PKGBUILD
endif

$(CHANGELOG_FILE): ./CHANGELOG
	@install -d "$(STAGING_DIR)"
	@cat ./CHANGELOG | awk -v version="^# v$(VERSION)" '$$0 ~ version {start=1}/^# v.*$$/ && $$0 !~ version {start=0}start' > "$(CHANGELOG_FILE)"

$(MAN_PAGES_DIR): $(MAN_PAGE_SRCS)
	@DATE="$$(date -uI)"; \
	for MANPAGE in $(MAN_PAGE_SRCS); do \
		NAME="$$(basename "$$MANPAGE")"; \
		SECTION="man1"; \
		case "$$NAME" in \
			*.2) SECTION="man2" ;; \
			*.3) SECTION="man3" ;; \
			*.4) SECTION="man4" ;; \
			*.5) SECTION="man5" ;; \
			*.6) SECTION="man6" ;; \
			*.7) SECTION="man7" ;; \
			*.8) SECTION="man8" ;; \
			*) SECTION="man1" ;; \
		esac; \
		install -d "$(MAN_PAGES_DIR)/$$SECTION/"; \
		cat "$$MANPAGE" | sed -e "s/__PLACEHOLDER_VERSION/$(VERSION)/g" -e "s/__PLACEHOLDER_DATE/$$DATE/g" > "$(MAN_PAGES_DIR)/$$SECTION/$$NAME"; \
	done

$(STAGING_TARGETS): $(TARGETS) $(PACKAGE_CONTENTS)
	@install -d "$(STAGING_DIR)"
	@for TARGET_ARCH in $(TARGETS); do \
		OUTDIR="$(STAGING_DIR)/nut_webgui_$(VERSION)_$$TARGET_ARCH"; \
		install -D "$(ARTIFACT_DIR)/$$TARGET_ARCH/nut_webgui" "$$OUTDIR/nut_webgui"; \
		for CONTENT in $(PACKAGE_CONTENTS); do \
			cp -r "$$CONTENT" "$$OUTDIR"; \
		done; \
	done

$(PACKAGE_TARS): $(STAGING_TARGETS)
	@install -d "$(PACKAGE_DIR)"
	@for TARGET in $(STAGING_TARGETS); do \
		DIR_NAME="$$(basename "$$TARGET")"; \
		OUTPUT_TAR="$(PACKAGE_DIR)/$$DIR_NAME.tar.gz"; \
		tar -czf "$$OUTPUT_TAR" -C "$(STAGING_DIR)" "$$DIR_NAME";  \
	done

$(MANIFEST_FILE): $(PACKAGE_TARS)
	@install -d "$(PACKAGE_DIR)"
	@echo "---" > "$(MANIFEST_FILE)";
	@for TAR_FILE in $(PACKAGE_TARS); do \
		echo "Filename: $$(basename "$$TAR_FILE")" >> "$(MANIFEST_FILE)"; \
		echo 'Version: $(VERSION)' >> "$(MANIFEST_FILE)"; \
		echo "Target: $$(basename "$$TAR_FILE" | sed -e 's/nut_webgui_$(VERSION)_//' -e 's/.tar.gz//')" >> "$(MANIFEST_FILE)"; \
		echo 'Revision: $(ANNOTATION_REVISION)' >> "$(MANIFEST_FILE)"; \
		sha1sum "$$TAR_FILE" | awk '{print "SHA1: "$$1}' >> $(MANIFEST_FILE) ; \
		sha256sum "$$TAR_FILE" | awk '{print "SHA256: "$$1}' >> $(MANIFEST_FILE) ; \
		md5sum "$$TAR_FILE" | awk '{print "MD5: "$$1}' >> $(MANIFEST_FILE) ; \
	  echo "---" >> "$(MANIFEST_FILE)"; \
	done

$(INSTALL_SCRIPT): ./dist/install.sh
	@install -D "./dist/install.sh" "$(INSTALL_SCRIPT)"

.PHONY: package
package: $(MANIFEST_FILE) $(INSTALL_SCRIPT)
