FJ_PUBLISH_DIR     := $(BIN_DIR)/fj_release
FJ_RELEASE_TARGETS := $(foreach TARGET,$(FJ_TARGETS),release-forgejo-$(TARGET))
FJ_RELEASE_ASSETS  := $(PACKAGE_TARS) $(MANIFEST_FILE) $(INSTALL_SCRIPT)

define release-forgejo-recipe =

ifndef FJ_$(1)_ACCESS_TOKEN
	ERROR_FJ_$(1)_ACCESS_TOKEN = $$(error FJ_$(1)_ACCESS_TOKEN environment variable is required for new forgejo release.)
endif

# Common headers
define FJ_$(1)_HEADERS =
-H "Authorization: token $$(FJ_$(1)_ACCESS_TOKEN)"
endef

FJ_$(1)_API_URI = $$(FJ_$(1)_URI)/api/v1/repos/$$(FJ_$(1)_OWNER)/$$(FJ_$(1)_REPO)

$(FJ_PUBLISH_DIR)/$(1)/release-request.json: $(CHANGELOG_FILE)
	@install -d $(FJ_PUBLISH_DIR)/$(1)
	@jq -cn \
		--rawfile body "$(CHANGELOG_FILE)" \
		'{body: $$$$body, draft: true, tag_name: "v$(VERSION)", name: "v$(VERSION)"}' > $(FJ_PUBLISH_DIR)/$(1)/release-request.json

.PHONY: release-forgejo-$(1)
release-forgejo-$(1): $(FJ_RELEASE_ASSETS) $(FJ_PUBLISH_DIR)/$(1)/release-request.json ; $$(ERROR_FJ_$(1)_ACCESS_TOKEN)
	@install -d $(FJ_PUBLISH_DIR)/$(1)
	@if curl -Lfs $$(FJ_$(1)_HEADERS) "$$(FJ_$(1)_API_URI)/releases/tags/v$(VERSION)" --out-null ; then \
		echo "v$(VERSION) tag is already exists on the forgejo $$(FJ_$(1)_OWNER)/$$(FJ_$(1)_REPO))"; \
		exit 1; \
	fi
	@curl -Lfs \
		-X POST \
		$$(FJ_$(1)_HEADERS) \
		-H "Accept: application/json" \
		-H "Content-Type: application/json" \
		"$$(FJ_$(1)_API_URI)/releases" \
		-d "@$(FJ_PUBLISH_DIR)/$(1)/release-request.json" | jq -cr '.id' > "$(FJ_PUBLISH_DIR)/$(1)/release.id"
	@RELEASE_ID="$$$$(cat "$(FJ_PUBLISH_DIR)/$(1)/release.id")"; \
	if [ -z "$$$$RELEASE_ID" ]; then \
		echo "Unable to create new release on $$(FJ_$(1)_URI)"; \
		exit 1; \
	fi; \
	for ASSET in $(FJ_RELEASE_ASSETS); do \
		NAME="$$$$(basename "$$$$ASSET")"; \
		curl -Lfs \
			-X POST \
			$$(FJ_$(1)_HEADERS) \
			-H "Content-Type: application/octet-stream" \
			--out-null \
			"$$(FJ_$(1)_API_URI)/releases/$$$$RELEASE_ID/assets?name=$$$$NAME" \
			--data-binary "@$$$$ASSET" && \
		echo "$$$$ASSET uploaded to $$(FJ_$(1)_URI)/$$(FJ_$(1)_OWNER)/$$(FJ_$(1)_REPO). Release Id: $$$$RELEASE_ID"; \
	done
endef

$(foreach TARGET,$(FJ_TARGETS), $(eval $(call release-forgejo-recipe,$(TARGET))))

.PHONY: release-forgejo
release-forgejo: $(FJ_RELEASE_TARGETS)
