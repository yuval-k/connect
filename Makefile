all: cross-build build target/layout.json

.PHONY: cross-build
cross-build:
	CARGO_BUILD_RUSTFLAGS="-C link_args=-L$(shell pwd)/lib/arm-linux-gnueabihf" cargo build --target arm-unknown-linux-gnueabihf

.PHONY: build
build:
	cargo build

target/layout.json: contrib/make_connect_layout.py
	python contrib/make_connect_layout.py > target/layout.json