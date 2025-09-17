# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust library called `failpoint` that provides simple fault injection capabilities. It's inspired by the `fault-injection` crate and is designed as a lightweight library for testing error conditions.

## Development Commands

### Building
```bash
cargo build
```

### Testing
```bash
cargo test
```

### Documentation
```bash
cargo doc --open
```

### Linting and Formatting
```bash
cargo clippy
cargo fmt
```

## Architecture

- **Library Structure**: Single-library crate with `lib.rs` as the main entry point
- **Core Components**:
  - `Reporter` type: Function pointer for reporting failpoint activations
  - `State` struct: Contains mutex-protected internal state for managing failpoints
  - Currently appears to be in early development with basic structure in place

## Development Notes

- This is a Rust library project using Cargo as the build system
- No external dependencies currently defined in Cargo.toml
- The codebase is minimal and appears to be in early development stages
- Licensed under MIT OR Apache-2.0