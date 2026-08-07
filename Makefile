# failpoint build entry point.
#
# failpoint is a plain Cargo crate — cargo is the real build system and
# nothing here replaces it. This file exists so failpoint answers the
# same four verbs as every other component in the SLDB family
# (`all`, `test`, `check`, `clean`), which is what lets a single
# orchestrator drive them all without special-casing this one.
#
# Every recipe below is exactly what .github/workflows/ci.yml runs, so
# a green `make check test` here means a green CI run. Keep them in
# step: if you change a command in one, change it in the other.
#
# `cargo build` / `cargo test` continue to work unchanged, and deleting
# this file breaks nothing.

CARGO ?= cargo

.PHONY: all build test check fmt clippy examples clean help

## all: build the library (default)
all: build

## build: compile the crate
build:
	$(CARGO) build

## test: unit tests plus doc tests
#
# --test-threads=1 is required, not a preference: failpoint's state is
# process-global, so concurrent tests trip each other's failpoints.
test:
	$(CARGO) test -- --test-threads=1
	$(CARGO) test --doc

## check: the lint gate — formatting and clippy, both fatal
#
# `cargo fmt` reads rustfmt.toml, which pins style_edition = "2024"
# while Cargo.toml declares edition = "2021". That is deliberate: the
# source is written in the 2024 import style, and without the pin
# rustfmt applies the 2021 style and rejects the tree.
check: fmt clippy

## fmt: fail if the tree is not formatted
fmt:
	$(CARGO) fmt -- --check

## clippy: fail on any warning
clippy:
	$(CARGO) clippy --all-targets -- -D warnings

## examples: run every example, asserting the conditional-compilation ones
#
# conditional_comp prints a different line depending on whether the
# `failpoint_enabled` default feature is on, so both directions are
# checked. The others just have to exit zero.
examples:
	@out=$$($(CARGO) run --quiet --example conditional_comp); \
	  echo "$$out"; \
	  case "$$out" in *"failpoint! is enabled"*) ;; \
	    *) echo "expected 'failpoint! is enabled'" >&2; exit 1;; esac
	@out=$$($(CARGO) run --quiet --example conditional_comp --no-default-features); \
	  echo "$$out"; \
	  case "$$out" in *"failpoint! is disabled"*) ;; \
	    *) echo "expected 'failpoint! is disabled'" >&2; exit 1;; esac
	$(CARGO) run --quiet --example test_codepath
	$(CARGO) run --quiet --example failpoint
	$(CARGO) run --quiet --example multiple_error_types

## clean: remove build artefacts
clean:
	$(CARGO) clean

## help: list targets
help:
	@grep -hE '^## [a-z]' $(MAKEFILE_LIST) | sed 's/^## /  /'
