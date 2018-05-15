all: cross-build build target/layout.json

TARGET=arm-linux-gnueabihf-

.PHONY: cross-build
cross-build:
	RUSTFLAGS="-C linker=$(TARGET)gcc" AR=$(TARGET)ar CC=$(TARGET)gcc cargo build --features ledscape --target arm-unknown-linux-gnueabihf
	RUSTFLAGS="-C linker=$(TARGET)gcc" AR=$(TARGET)ar CC=$(TARGET)gcc cargo build --release --features ledscape --target arm-unknown-linux-gnueabihf

.PHONY: all_pru_templates
all_pru_templates:
	git submodule update --init
	$(MAKE) -C lib/LEDscape all_pru_templates
	cp lib/LEDscape/pru/bin/*.bin ./lib/bin/

.PHONY: build
build:
	cargo build --features gui

#target/layout.json: contrib/make_connect_layout.py
#	python contrib/make_connect_layout.py > target/layout.json

target/layout.json: contrib/Pixel_Coordinates.xlsx contrib/xlsx_layout.py
	python contrib/xlsx_layout.py contrib/Pixel_Coordinates.xlsx  > target/layout.json