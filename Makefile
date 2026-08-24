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

.PHONY: all build test check fmt check_fmt clippy examples clean help

## all: build the library (default)
all: build

## build: compile the crate
build:
	$(CARGO) build

## test: unit tests via nextest, plus doc tests
#
# --test-threads=1 USED TO BE REQUIRED HERE, and no longer is. The
# reason it was needed — failpoint's state is process-global, so
# concurrent tests tripped each other's failpoints — is exactly what
# nextest removes: it runs each test in its own PROCESS, so there is no
# shared global left to contend over. Verified rather than assumed: 12
# consecutive default-parallelism runs, 9/9 passing each time.
#
# If you ever see a flake here, that reasoning is the thing to re-check
# before adding a concurrency flag back.
#
# The `--doc` line is NOT optional: nextest does not run doctests, and
# this crate has 11. It also stops them being run twice — a plain
# `cargo test` already included them, so the old pair of lines ran the
# doctests once each.
test:
	$(CARGO) nextest run
	$(CARGO) test --doc

## check: the lint gate — formatting and clippy, both fatal
#
# `cargo fmt` reads rustfmt.toml, which pins style_edition = "2024"
# while Cargo.toml declares edition = "2021". That is deliberate: the
# source is written in the 2024 import style, and without the pin
# rustfmt applies the 2021 style and rejects the tree.
check: check_fmt clippy

## fmt: reformat the tree in place
#
# `fmt` REWRITES and `check_fmt` reports, the same way round in every
# component here. A gate must not edit your working tree, so `check`
# depends on the reporting one.
fmt:
	$(CARGO) fmt

## check_fmt: fail if the tree is not formatted
check_fmt:
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

## actionlint: lint this component's GitHub Actions workflows
#
# Self-contained on purpose. This file is exported to the component's
# own repository, where there is no monorepo root Makefile to fall back
# on, so `make actionlint` has to work standing alone.
#
# `wildcard` makes it a no-op rather than an error when there are no
# workflows here, so the target exists uniformly across every component
# and the root can call it without knowing which have any.
#
# WHY IT EXISTS. GitHub's workflow parser is stricter than YAML and
# everything below it is silent about the difference: on 2026-08-20 a
# comment containing the literal expression braces was valid YAML,
# passed every structural check, and was rejected outright by GitHub
# with "workflow file issue" and zero jobs.
ACTIONLINT ?= actionlint
# See the root Makefile: disabled because actionlint runs these only
# if they are on PATH, so results would vary by machine.
ACTIONLINT_FLAGS ?= -shellcheck= -pyflakes=
ACTIONLINT_FILES := $(wildcard .github/workflows/*.yml)

actionlint:
ifeq ($(ACTIONLINT_FILES),)
	@echo "actionlint: no workflows in $(CURDIR) — nothing to do"
else
	$(ACTIONLINT) $(ACTIONLINT_FLAGS) $(ACTIONLINT_FILES)
endif
.PHONY: actionlint
