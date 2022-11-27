IMAGE=ghcr.io/cakemanny/k8s-secret-check
BRANCH=$(shell git branch --show-current)
REV=$(shell git rev-parse --short=10 HEAD)
ARCH=$(shell uname -m)

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
