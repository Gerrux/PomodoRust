.PHONY: all build release fast debug clean run test dist install help

# Configuration
BINARY_NAME := pomodorust
DIST_DIR := dist
VERSION := $(shell grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')

# Detect OS
UNAME_S := $(shell uname -s 2>/dev/null || echo Windows)
ifeq ($(UNAME_S),Darwin)
    PLATFORM := macos
    INSTALL_DIR := /usr/local/bin
else ifeq ($(UNAME_S),Linux)
    PLATFORM := linux
    INSTALL_DIR := /usr/local/bin
else
    PLATFORM := windows
    INSTALL_DIR := $(USERPROFILE)/bin
endif

# Default target
all: release

# Build targets
build: release

release:
	@echo "Building release..."
	cargo build --release
	@$(MAKE) --no-print-directory _copy_to_dist PROFILE=release

fast:
	@echo "Building release-fast..."
	cargo build --profile release-fast
	@$(MAKE) --no-print-directory _copy_to_dist PROFILE=release-fast

debug:
	@echo "Building debug..."
	cargo build
	@$(MAKE) --no-print-directory _copy_to_dist PROFILE=debug

# Internal: copy binary to dist
_copy_to_dist:
	@mkdir -p $(DIST_DIR)
ifeq ($(PROFILE),debug)
	@cp target/debug/$(BINARY_NAME)* $(DIST_DIR)/ 2>/dev/null || true
else
	@cp target/$(PROFILE)/$(BINARY_NAME)* $(DIST_DIR)/ 2>/dev/null || true
endif
	@echo "Output: $(DIST_DIR)/"

# Clean
clean:
	cargo clean
	rm -rf $(DIST_DIR)/

# Run
run:
	cargo run

run-release:
	cargo run --release

# Test
test:
	cargo test

test-verbose:
	cargo test -- --nocapture

# Check and lint
check:
	cargo check

clippy:
	cargo clippy -- -D warnings

fmt:
	cargo fmt

fmt-check:
	cargo fmt -- --check

# Full CI check
ci: fmt-check clippy test build

# Distribution with compression
dist: release
	@echo "Creating distribution package..."
	@mkdir -p $(DIST_DIR)
ifeq ($(PLATFORM),windows)
	@if command -v upx >/dev/null 2>&1; then \
		echo "Compressing with UPX..."; \
		upx --best --lzma target/release/$(BINARY_NAME).exe -f -o $(DIST_DIR)/$(BINARY_NAME).exe 2>/dev/null || \
		cp target/release/$(BINARY_NAME).exe $(DIST_DIR)/; \
	else \
		cp target/release/$(BINARY_NAME).exe $(DIST_DIR)/; \
	fi
else
	@if command -v upx >/dev/null 2>&1; then \
		echo "Compressing with UPX..."; \
		upx --best --lzma target/release/$(BINARY_NAME) -f -o $(DIST_DIR)/$(BINARY_NAME)-$(PLATFORM) 2>/dev/null || \
		cp target/release/$(BINARY_NAME) $(DIST_DIR)/$(BINARY_NAME)-$(PLATFORM); \
	else \
		cp target/release/$(BINARY_NAME) $(DIST_DIR)/$(BINARY_NAME)-$(PLATFORM); \
	fi
	@if command -v strip >/dev/null 2>&1; then \
		strip $(DIST_DIR)/$(BINARY_NAME)-$(PLATFORM) 2>/dev/null || true; \
	fi
endif
	@echo "Distribution ready in $(DIST_DIR)/"
	@ls -lh $(DIST_DIR)/

# Install locally
install: release
ifeq ($(PLATFORM),windows)
	@mkdir -p "$(INSTALL_DIR)"
	@cp target/release/$(BINARY_NAME).exe "$(INSTALL_DIR)/"
	@echo "Installed to $(INSTALL_DIR)/$(BINARY_NAME).exe"
else
	@mkdir -p $(INSTALL_DIR)
	@cp target/release/$(BINARY_NAME) $(INSTALL_DIR)/
	@chmod +x $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "Installed to $(INSTALL_DIR)/$(BINARY_NAME)"
endif

uninstall:
ifeq ($(PLATFORM),windows)
	@rm -f "$(INSTALL_DIR)/$(BINARY_NAME).exe"
else
	@rm -f $(INSTALL_DIR)/$(BINARY_NAME)
endif
	@echo "Uninstalled $(BINARY_NAME)"

# Cross-compilation targets
build-windows:
	cargo build --release --target x86_64-pc-windows-msvc

build-linux:
	cargo build --release --target x86_64-unknown-linux-gnu

build-macos:
	cargo build --release --target x86_64-apple-darwin

build-macos-arm:
	cargo build --release --target aarch64-apple-darwin

# Build all platforms (requires cross-compilation setup)
build-all: build-windows build-linux build-macos build-macos-arm

# Size analysis
size: release
	@echo "Binary size analysis:"
	@ls -lh target/release/$(BINARY_NAME)* 2>/dev/null || ls -lh target/release/$(BINARY_NAME)
ifeq ($(PLATFORM),linux)
	@if command -v bloaty >/dev/null 2>&1; then \
		bloaty target/release/$(BINARY_NAME); \
	fi
endif

# Help
help:
	@echo "pomodorust build system"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Build targets:"
	@echo "  release      Build optimized for size (default)"
	@echo "  fast         Build optimized for speed"
	@echo "  debug        Build with debug info"
	@echo "  dist         Build and package for distribution"
	@echo ""
	@echo "Development:"
	@echo "  run          Run debug build"
	@echo "  run-release  Run release build"
	@echo "  test         Run tests"
	@echo "  check        Quick syntax check"
	@echo "  clippy       Run linter"
	@echo "  fmt          Format code"
	@echo "  ci           Run full CI checks"
	@echo ""
	@echo "Installation:"
	@echo "  install      Install to $(INSTALL_DIR)"
	@echo "  uninstall    Remove installation"
	@echo ""
	@echo "Other:"
	@echo "  clean        Remove build artifacts"
	@echo "  size         Show binary size info"
	@echo "  help         Show this help"
