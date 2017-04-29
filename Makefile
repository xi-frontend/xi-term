.PHONY: all build test fmt clean

FEATURES =
TRAVIS_RUST_VERSION ?= stable
ifeq (nightly, $(TRAVIS_RUST_VERSION))
	FEATURES=clippy
endif

all: build

build: src
	cargo build --features "$(FEATURES)"

test:
	cargo test --features "$(FEATURES)"

fmt:
	cargo fmt

clean:
	rm -rf target
