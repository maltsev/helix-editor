PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
CARGO ?= cargo
CARGO_TARGET_DIR ?= target
SUDO ?= sudo
BUILD_DIR := $(if $(filter /%,$(CARGO_TARGET_DIR)),$(CARGO_TARGET_DIR),$(CURDIR)/$(CARGO_TARGET_DIR))

.PHONY: build install link dev

# Use the optimized release profile and this checkout's runtime files.
build:
	HELIX_DEFAULT_RUNTIME="$(CURDIR)/runtime" $(CARGO) build --profile opt --locked --package helix-term --bin hx --target-dir "$(CARGO_TARGET_DIR)"

# Build a local debug executable, reusing this checkout's compiled grammars.
dev:
	HELIX_DISABLE_AUTO_GRAMMAR_BUILD=1 HELIX_DEFAULT_RUNTIME="$(CURDIR)/runtime" $(CARGO) build --profile dev --locked --package helix-term --bin hx --target-dir "$(CARGO_TARGET_DIR)"

# Build as the current user; elevate only the installation commands.
install: build
	$(SUDO) install -d "$(BINDIR)"
	if test -L "$(BINDIR)/hx"; then $(SUDO) rm -f "$(BINDIR)/hx"; fi
	$(SUDO) install -m 755 "$(CARGO_TARGET_DIR)/opt/hx" "$(BINDIR)/hx"

link: build
	$(SUDO) install -d "$(BINDIR)"
	$(SUDO) ln -sfn "$(BUILD_DIR)/opt/hx" "$(BINDIR)/hx"
