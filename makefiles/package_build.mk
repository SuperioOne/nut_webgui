PACKAGE_DIR      := $(BIN_DIR)/package
STAGING_DIR      := $(BIN_DIR)/staging
PACKAGE_TARS     := $(foreach TARGET,$(TARGETS),$(PACKAGE_DIR)/nut_webgui_$(VERSION)_$(TARGET).tar.gz)
ARTIFACT_FILES   := $(foreach TARGET,$(TARGETS),$(ARTIFACT_DIR)/$(TARGET)/nut_webgui)
CHANGELOG_FILE   := $(STAGING_DIR)/CHANGELOG
INSTALL_SCRIPT   := $(PACKAGE_DIR)/install.sh
MANIFEST_FILE    := $(PACKAGE_DIR)/MANIFEST
MAN_OUT_DIR      := $(STAGING_DIR)/man
MAN_PAGE_SRC     := $(wildcard ./dist/man/*)
SERVICE_SRC      := $(wildcard ./dist/service/*) \
										$(wildcard ./dist/service/**/*)
STAGING_TARGETS  := $(foreach TARGET,$(TARGETS),$(STAGING_DIR)/nut_webgui_$(VERSION)_$(TARGET))
PACKAGE_CONTENTS := $(CHANGELOG_FILE) \
										$(MAN_OUT_DIR) \
										./LICENSE \
										./dist/config.toml \
										./dist/service \
										./dist/users.toml

# Parameters:
# 1: Target architecture
define tar_package_recipe =
$(STAGING_DIR)/nut_webgui_$(VERSION)_$(1): $(ARTIFACT_DIR)/$(1)/nut_webgui $(PACKAGE_CONTENTS) $(SERVICE_SRC)
	@install -p -D "$(ARTIFACT_DIR)/$(1)/nut_webgui" "$(STAGING_DIR)/nut_webgui_$(VERSION)_$(1)/nut_webgui"
	@cp -p -r $(PACKAGE_CONTENTS) "$(STAGING_DIR)/nut_webgui_$(VERSION)_$(1)"

$(PACKAGE_DIR)/nut_webgui_$(VERSION)_$(1).tar.gz: $(STAGING_DIR)/nut_webgui_$(VERSION)_$(1)
	@install -d "$(PACKAGE_DIR)"
	@tar -czf "$(PACKAGE_DIR)/nut_webgui_$(VERSION)_$(1).tar.gz" \
		-C "$(STAGING_DIR)" \
		"nut_webgui_$(VERSION)_$(1)"
endef

$(CHANGELOG_FILE): ./CHANGELOG
	@install -d "$(STAGING_DIR)"
	@cat ./CHANGELOG | awk -v version="^## v$(VERSION)" '$$0 ~ version {start=1}/^## v.*$$/ && $$0 !~ version {start=0}start' > "$(CHANGELOG_FILE)"

$(MAN_OUT_DIR): $(MAN_PAGE_SRC)
	@DATE="$$(date -uI)"; \
		for MANPAGE in $(MAN_PAGE_SRC); do \
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
			install -d "$(MAN_OUT_DIR)/$$SECTION/"; \
			cat "$$MANPAGE" | sed -e "s/__PLACEHOLDER_VERSION/$(VERSION)/g" -e "s/__PLACEHOLDER_DATE/$$DATE/g" > "$(MAN_OUT_DIR)/$$SECTION/$$NAME"; \
		done

$(foreach TARGET,$(TARGETS),$(eval $(call tar_package_recipe,$(TARGET))))

$(MANIFEST_FILE): $(PACKAGE_TARS)
	@install -d "$(PACKAGE_DIR)"
	@set -e; \
		echo "---" > "$(MANIFEST_FILE)"; \
		for TAR_FILE in $(PACKAGE_TARS); do \
			echo "Filename: $$(basename "$$TAR_FILE")" >> "$(MANIFEST_FILE)"; \
			echo 'Version: $(VERSION)' >> "$(MANIFEST_FILE)"; \
			echo "Fullname: $$(basename "$$TAR_FILE" | sed -e 's/.tar.gz//')" >> "$(MANIFEST_FILE)"; \
			echo "Target: $$(basename "$$TAR_FILE" | sed -e 's/nut_webgui_$(VERSION)_//' -e 's/.tar.gz//')" >> "$(MANIFEST_FILE)"; \
			echo 'Revision: $(ANNOTATION_REVISION)' >> "$(MANIFEST_FILE)"; \
			sha1sum "$$TAR_FILE" | awk '{print "SHA1: "$$1}' >> $(MANIFEST_FILE) ; \
			sha256sum "$$TAR_FILE" | awk '{print "SHA256: "$$1}' >> $(MANIFEST_FILE) ; \
			md5sum "$$TAR_FILE" | awk '{print "MD5: "$$1}' >> $(MANIFEST_FILE) ; \
			echo "---" >> "$(MANIFEST_FILE)"; \
		done

$(INSTALL_SCRIPT): ./dist/install.sh
	@install -p -D "./dist/install.sh" "$(INSTALL_SCRIPT)"

.PHONY: package
package: $(MANIFEST_FILE) $(INSTALL_SCRIPT)
