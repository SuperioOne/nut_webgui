PUBLISH_IMAGE_TARGETS := $(foreach REGISTRY,$(OCI_TARGETS),publish-images-$(REGISTRY))

define publish_image_recipe =

ifndef OCI_$(1)_USERNAME
	ERROR_OCI_$(1)_USERNAME = $$(error OCI_$(1)_USERNAME environment variable is required for publishing images.)
endif

ifndef OCI_$(1)_ACCESS_TOKEN
	ERROR_OCI_$(1)_ACCESS_TOKEN = $$(error OCI_$(1)_ACCESS_TOKEN environment variable is required for publishing images.)
endif

ifndef OCI_$(1)_TLS_VERIFY
	OCI_$(1)_TLS_VERIFY = true
endif

.PHONY: publish-images-$(1)
publish-images-$(1): build-images ; $$(ERROR_OCI_$(1)_ACCESS_TOKEN) $$(ERROR_OCI_$(1)_USERNAME)
	buildah manifest push --all \
		--tls-verify=$$(OCI_$(1)_TLS_VERIFY) \
		--creds '$$(OCI_$(1)_USERNAME):$$(OCI_$(1)_ACCESS_TOKEN)' \
		'nut_webgui:$(VERSION)' \
		'docker://$(OCI_$(1)_URI)/nut_webgui:latest';
	buildah manifest push --all \
		--tls-verify=$$(OCI_$(1)_TLS_VERIFY) \
		--creds '$$(OCI_$(1)_USERNAME):$$(OCI_$(1)_ACCESS_TOKEN)' \
		'nut_webgui:$(VERSION)' \
		'docker://$(OCI_$(1)_URI)/nut_webgui:$(VERSION)';
	buildah manifest push --all \
		--tls-verify=$$(OCI_$(1)_TLS_VERIFY) \
		--creds '$$(OCI_$(1)_USERNAME):$$(OCI_$(1)_ACCESS_TOKEN)' \
		'nut_webgui:$(VERSION)' \
		'docker://$(OCI_$(1)_URI)/nut_webgui:$(VERSION_MAJOR).$(VERSION_MINOR)';
	for TAG in $(IMAGE_TARGETS); do \
		buildah push \
		--tls-verify=$$(OCI_$(1)_TLS_VERIFY) \
		--creds '$$(OCI_$(1)_USERNAME):$$(OCI_$(1)_ACCESS_TOKEN)' \
		"nut_webgui:$(VERSION)-$$$$TAG" \
		"docker://$(OCI_$(1)_URI)/nut_webgui:$(VERSION)-$$$$TAG"; \
	done
ifdef ENABLE_X86_64_V3_MUSL
	buildah push \
		--tls-verify=$$(OCI_$(1)_TLS_VERIFY) \
		--creds '$$(OCI_$(1)_USERNAME):$$(OCI_$(1)_ACCESS_TOKEN)' \
		'nut_webgui:$(VERSION)-amd64-v3' \
		'docker://$(OCI_$(1)_URI)/nut_webgui:latest-amd64-v3';
endif
ifdef ENABLE_X86_64_V4_MUSL
	buildah push \
		--tls-verify=$$(OCI_$(1)_TLS_VERIFY) \
		--creds '$$(OCI_$(1)_USERNAME):$$(OCI_$(1)_ACCESS_TOKEN)' \
		'nut_webgui:$(VERSION)-amd64-v4' \
		'docker://$(OCI_$(1)_URI)/nut_webgui:latest-amd64-v4';
endif
endef

$(foreach REGISTRY,$(OCI_TARGETS),$(eval $(call publish_image_recipe,$(REGISTRY))))

.PHONY: publish-images
publish-images: $(PUBLISH_IMAGE_TARGETS)
