PACKAGE_DIR      := $(BIN_DIR)/packages
STAGING_DIR      := $(BIN_DIR)/staging
STAGING_TARGETS  := $(foreach TARGET,$(TARGETS),$(STAGING_DIR)/nut_webgui_$(VERSION)_$(TARGET))
PACKAGE_TARS     := $(foreach TARGET,$(TARGETS),$(PACKAGE_DIR)/nut_webgui_$(VERSION)_$(TARGET).tar.gz)
CHANGELOG_FILE   := $(STAGING_DIR)/CHANGELOG
INSTALL_SCRIPT   := $(PACKAGE_DIR)/install.sh
MANIFEST_FILE    := $(PACKAGE_DIR)/MANIFEST
PKGBUILD_ENABLED := $(and $(ENABLE_ARMV7_MUSLEABI),$(ENABLE_AARCH64_GNU),$(ENABLE_X86_64_GNU))
PACKAGE_CONTENTS := $(TARGETS) \
										$(CHANGELOG_FILE) \
										./LICENSE \
										./dist/config.toml \
										./dist/openrc/nut_webgui \
										./dist/runit/log/run \
										./dist/runit/run \
										./dist/systemd/nut_webgui.service \
										./dist/users.toml

ifdef PKGBUILD_ENABLED
PKGBUILD_SCRIPT  := $(PACKAGE_DIR)/PKGBUILD
endif

$(CHANGELOG_FILE): ./CHANGELOG
	@install -d "$(STAGING_DIR)"
	@cat ./CHANGELOG | awk -v version="^# v$(VERSION)" '$$0 ~ version {start=1;next}/^# v.*$$/{start=0}start' > "$(CHANGELOG_FILE)"

$(STAGING_TARGETS): $(PACKAGE_CONTENTS)
	@install -d "$(STAGING_DIR)"
	@for TARGET_ARCH in $(TARGETS); do \
		OUTDIR="$(STAGING_DIR)/nut_webgui_$(VERSION)_$$TARGET_ARCH"; \
		install -D "$(ARTIFACT_DIR)/$$TARGET_ARCH/nut_webgui" "$$OUTDIR/nut_webgui"; \
		install -D "./dist/config.toml" "$$OUTDIR/config.toml"; \
		install -D "./dist/users.toml" "$$OUTDIR/users.toml"; \
		install -D "./LICENSE" "$$OUTDIR/LICENSE"; \
		install -D "$(STAGING_DIR)/CHANGELOG" "$$OUTDIR/CHANGELOG"; \
		install -D "./dist/systemd/nut_webgui.service" "$$OUTDIR/service/systemd/nut_webgui.service"; \
		install -D "./dist/openrc/nut_webgui" "$$OUTDIR/service/openrc/nut_webgui"; \
		install -D "./dist/runit/run" "$$OUTDIR/service/runit/run"; \
		install -D "./dist/runit/log/run" "$$OUTDIR/service/runit/log/run"; \
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
	install -D "./dist/install.sh" "$(INSTALL_SCRIPT)"

.PHONY: package
package: $(MANIFEST_FILE) $(INSTALL_SCRIPT)
