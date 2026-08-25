CONTAINER_DIR        := $(BIN_DIR)/container
OCI_IMAGE_NAMESPACE  := org.opencontainers.image
IMAGE_TEMPLATE       := ./dist/container/Dockerfile.template
IMAGE_DEPENDENCIES   := ./dist/container/server_init.sh \
												./dist/config.toml \
												./dist/users.toml

define ANNOTATIONS =
--annotation "$(OCI_IMAGE_NAMESPACE).authors=$(ANNOTATION_AUTHORS)" \
--annotation "$(OCI_IMAGE_NAMESPACE).description=$(ANNOTATION_DESCRIPTION)" \
--annotation "$(OCI_IMAGE_NAMESPACE).documentation=$(ANNOTATION_DOCUMENTATION)" \
--annotation "$(OCI_IMAGE_NAMESPACE).licenses=$(ANNOTATION_LICENSES)" \
--annotation "$(OCI_IMAGE_NAMESPACE).revision=$(ANNOTATION_REVISION)" \
--annotation "$(OCI_IMAGE_NAMESPACE).source=$(ANNOTATION_SOURCE)" \
--annotation "$(OCI_IMAGE_NAMESPACE).title=$(ANNOTATION_TITLE)" \
--annotation "$(OCI_IMAGE_NAMESPACE).url=$(ANNOTATION_URL)" \
--annotation "$(OCI_IMAGE_NAMESPACE).version=$(ANNOTATION_VERSION)"
endef

# Parameters
# 1: Image tag
# 2: Binary artifact directory name
# 3: Architecture
# 4: Variant
define container_image_rule =
$(CONTAINER_DIR)/$(1).Dockerfile: $(2) $(IMAGE_DEPENDENCIES) $(IMAGE_TEMPLATE)
	@install -d "$(CONTAINER_DIR)"
	@export PLACEHOLDER_EXE_DIR="$(ARTIFACT_DIR)/$(2)"; \
		cat "$(IMAGE_TEMPLATE)" | envsubst > "$(CONTAINER_DIR)/$(1).Dockerfile"

$(CONTAINER_DIR)/$(1).info: $(CONTAINER_DIR)/$(1).Dockerfile
	@install -d "$(CONTAINER_DIR)"
	@REBUILD=; \
	IMAGE_ID="$$$$(buildah inspect --type image -f "{{.FromImageID}}" "nut_webgui:$(VERSION)-$(1)")"; \
		if [ -e "$(CONTAINER_DIR)/$(1).info" ]; then \
			EXISTING_ID="$$$$(cat "$(CONTAINER_DIR)/$(1).info")"; \
			if [ "$$$$EXISTING_ID" != "$$$$IMAGE_ID" ]; then \
				REBUILD=true; \
			fi; \
		else \
			REBUILD=true; \
		fi; \
		if [ -n "$$$$REBUILD" ]; then \
			buildah build \
				--arch "$(3)" \
				--variant "$(4)" \
				$(ANNOTATIONS) \
				-t "nut_webgui:$(VERSION)-$(1)" \
				-f "$(CONTAINER_DIR)/$(1).Dockerfile"; \
			buildah inspect \
				--type image \
				-f "{{.FromImageID}}" \
				"nut_webgui:$(VERSION)-$(1)" > "$(CONTAINER_DIR)/$(1).info"; \
		fi;

.PHONY: $(1)
$(1): $(CONTAINER_DIR)/$(1).info

endef

ifdef ENABLE_X86_64_MUSL
$(eval $(call container_image_rule,amd64,x86-64-musl,amd64,))
endif

ifdef ENABLE_X86_64_V3_MUSL
$(eval $(call container_image_rule,amd64-v3,x86-64-v3-musl,amd64,v3))
endif

ifdef ENABLE_X86_64_V4_MUSL
$(eval $(call container_image_rule,amd64-v4,x86-64-v4-musl,amd64,v4))
endif

ifdef ENABLE_AARCH64_MUSL
$(eval $(call container_image_rule,arm64,aarch64-musl,arm64,v8))
endif

ifdef ENABLE_ARMV7_MUSLEABI
$(eval $(call container_image_rule,armv7,armv7-musleabi,arm,v7))
endif

ifdef ENABLE_ARMV6_MUSLEABI
$(eval $(call container_image_rule,armv6,armv6-musleabi,arm,v6))
endif

ifdef ENABLE_RISCV_MUSL
$(eval $(call container_image_rule,riscv64,riscv64gc-musl,riscv64,))
endif

.PHONY: build-images
build-images: $(IMAGE_TARGETS)
	@if buildah manifest exists "nut_webgui:$(VERSION)"; then \
		buildah manifest rm "nut_webgui:$(VERSION)"; \
	fi
	@buildah manifest create \
		$(ANNOTATIONS) \
		"nut_webgui:$(VERSION)"
	@for TAG in $(IMAGE_TARGETS); do \
		buildah manifest add "nut_webgui:$(VERSION)" "nut_webgui:$(VERSION)-$$TAG"; \
	done
