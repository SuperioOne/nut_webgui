GH_PUBLISH_DIR     := $(BIN_DIR)/gh_release
GH_RELEASE_TARGETS := $(foreach TARGET,$(GH_TARGETS),release-github-$(TARGET))
GH_RELEASE_ASSETS  := $(PACKAGE_TARS) $(MANIFEST_FILE) $(INSTALL_SCRIPT)

define release-github-recipe =

ifndef GH_$(1)_ACCESS_TOKEN
	ERROR_GH_$(1)_ACCESS_TOKEN = $$(error GH_$(1)_ACCESS_TOKEN environment variable is required for new Github release.)
endif

# Common headers
define GH_$(1)_HEADERS = 
-H "Authorization: Bearer $$(GH_$(1)_ACCESS_TOKEN)"
endef

GH_$(1)_API_URI = https://api.github.com/repos/$$(GH_$(1)_OWNER)/$$(GH_$(1)_REPO)
GH_$(1)_UPLOAD_URI = https://uploads.github.com/repos/$$(GH_$(1)_OWNER)/$$(GH_$(1)_REPO)

$(GH_PUBLISH_DIR)/$(1)/release-request.json: $(CHANGELOG_FILE)
	@install -d $(GH_PUBLISH_DIR)/$(1)
	@jq -cn \
		--rawfile body "$(CHANGELOG_FILE)" \
		'{body: $$$$body, draft: true, tag_name: "v$(VERSION)", name: "v$(VERSION)"}' > $(GH_PUBLISH_DIR)/$(1)/release-request.json

.PHONY: release-github-$(1)
release-github-$(1): $(GH_RELEASE_ASSETS) $(GH_PUBLISH_DIR)/$(1)/release-request.json ; $$(ERROR_GH_$(1)_ACCESS_TOKEN)
	@install -d $(GH_PUBLISH_DIR)/$(1)
	@if curl -Lfs $$(GH_$(1)_HEADERS) "$$(GH_$(1)_API_URI)/releases/tags/v$(VERSION)" --out-null ; then \
		echo "v$(VERSION) tag is already exists on the Github $$(GH_$(1)_OWNER)/$$(GH_$(1)_REPO))"; \
		exit 1; \
	fi
	@curl -Lfs \
		-X POST \
		$$(GH_$(1)_HEADERS) \
		-H "Accept: application/vnd.github+json" \
		-H "Content-Type: application/json" \
		"$$(GH_$(1)_API_URI)/releases" \
		-d "@$(GH_PUBLISH_DIR)/$(1)/release-request.json" | jq -cr '.id' > "$(GH_PUBLISH_DIR)/$(1)/release.id"
	@RELEASE_ID="$$$$(cat "$(GH_PUBLISH_DIR)/$(1)/release.id")"; \
	if [ -z "$$$$RELEASE_ID" ]; then \
		echo "Unable to create new release on $$(GH_$(1)_OWNER)/$$(GH_$(1)_REPO)"; \
		exit 1; \
	fi; \
	for ASSET in $(GH_RELEASE_ASSETS); do \
		NAME="$$$$(basename "$$$$ASSET")"; \
		curl -Lfs \
			-X POST \
			$$(GH_$(1)_HEADERS) \
			-H "Content-Type: application/octet-stream" \
			--out-null \
			"$$(GH_$(1)_UPLOAD_URI)/releases/$$$$RELEASE_ID/assets?name=$$$$NAME" \
			--data-binary "@$$$$ASSET" && \
		echo "$$$$ASSET uploaded to $$(GH_$(1)_OWNER)/$$(GH_$(1)_REPO). Release Id: $$$$RELEASE_ID"; \
	done
endef

$(foreach TARGET,$(GH_TARGETS), $(eval $(call release-github-recipe,$(TARGET))))

.PHONY: release-github
release-github: $(GH_RELEASE_TARGETS)
