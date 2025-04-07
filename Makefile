#
# Copyright 2023, Colias Group, LLC
#
# SPDX-License-Identifier: BSD-2-Clause
#

# Configurable variables with default values, can be overridden when running `make`
MICROKIT_SDK ?= /home/li/Sel4/microkit-sdk-2.0.1
BUILD ?= build
BOARD ?= odroidc4

# Derived variables
BUILD_DIR := $(BUILD)/$(BOARD)
MICROKIT_BOARD := $(BOARD)
MICROKIT_TOOL := $(MICROKIT_SDK)/bin/microkit
MICROKIT_CONFIG ?= debug
# Allow for different build configurations (default is debug)
microkit_sdk_config_dir := $(MICROKIT_SDK)/board/$(MICROKIT_BOARD)/$(MICROKIT_CONFIG)
sel4_include_dirs := $(microkit_sdk_config_dir)/include

# Target output
TARGET_ELF := $(BUILD_DIR)/target/aarch64-sel4-microkit-minimal/debug/sdmmc_driver.elf

# Default target if none is provided
.PHONY: none
none:
	@echo "No target specified. Use 'make build' or other targets."

# Clean target
.PHONY: clean
clean:
	@echo "Cleaning build directory..."
	rm -rf $(BUILD)
	@echo "Clean complete."

# Ensure build directory exists
$(BUILD_DIR):
	@echo "Creating build directory $(BUILD_DIR)..."
	mkdir -p $@

# Main build target
$(TARGET_ELF): $(BUILD_DIR)
	@echo "Building sdmmc_driver.elf for board $(MICROKIT_BOARD)..."
	@echo "MICROKIT SDK config directory: $(microkit_sdk_config_dir)"
	@echo "SEl4 include directories: $(sel4_include_dirs)"
	@SEL4_INCLUDE_DIRS=$(abspath $(sel4_include_dirs)) \
	cargo build \
		-Z build-std=core,alloc,compiler_builtins \
		-Z build-std-features=compiler-builtins-mem \
		--target-dir $(BUILD_DIR)/target \
		--target support/targets/aarch64-sel4-microkit-minimal.json
	@echo "Build complete: $(TARGET_ELF)"

IMAGE_FILE := loader.img
REPORT_FILE := report.txt

# Build target
.PHONY: build
build: $(IMAGE_FILE)
	@echo "Build finished successfully."

$(IMAGE_FILE) $(REPORT_FILE): $(TARGET_ELF) ./mmc.system
	$(MICROKIT_TOOL) ./mmc.system --search-path $(BUILD_DIR)/target/aarch64-sel4-microkit-minimal/debug --board $(MICROKIT_BOARD) --config $(MICROKIT_CONFIG) -o $(IMAGE_FILE) -r $(REPORT_FILE)