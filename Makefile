IMAGE=ghcr.io/cakemanny/k8s-secret-check
BRANCH=$(shell git branch --show-current)
REV=$(shell git rev-parse --short=10 HEAD)

ARCH := $(shell uname -m)
ifeq "$(ARCH)" "x86_64"
ARCH := amd64
endif
ifeq "$(ARCH)" "aarch64"
ARCH := arm64
endif

ifeq "$(shell git status --short)" ""
DIRTY=
else
DIRTY=-dirty
endif

# Specify VERSION to override
VERSION=$(BRANCH)-$(REV)-$(ARCH)$(DIRTY)

ifeq "$(ARCH)" ""

.PHONY: default
default:
	docker buildx build \
		--platform linux/amd64,linux/arm64 \
		-t $(IMAGE):$(VERSION) \
		--label org.opencontainers.image.revision=$(REV)$(DIRTY) \
		--push \
		.

endif

.PHONY: docker
docker:
	docker build . \
		--platform linux/$(ARCH) \
		-t $(IMAGE):$(VERSION) \
		--label org.opencontainers.image.revision=$(REV)$(DIRTY)

.PHONY: push
push:
	docker push $(IMAGE):$(VERSION)

.PHONY: manifest
manifest:
	docker manifest create $(IMAGE):$(BRANCH)-$(REV) \
		$(IMAGE):$(BRANCH)-$(REV)-amd64 \
		$(IMAGE):$(BRANCH)-$(REV)-arm64
	docker manifest push $(IMAGE):$(BRANCH)-$(REV)
