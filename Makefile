BRANCH=$(shell git branch --show-current)
REV=$(shell git rev-parse --short=10 HEAD)

ifeq "$(shell git status --short)" ""
DIRTY=
else
DIRTY=-dirty
endif

# Specify VERSION to override
VERSION=$(BRANCH)-$(REV)$(DIRTY)

docker:
	docker build . \
		-t k8s-secret-check:$(VERSION) \
		--label org.opencontainers.image.revision=$(REV)$(DIRTY)
