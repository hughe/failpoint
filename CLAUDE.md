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

Because the library uses a thread safe singleton for holding it's
state, running the tests in parallel can have unpredictable results.

```bash
cargo test -- --test-threads=1 
```

### Examples

Run an example

```bash
cargo run --example EXAMPLE-NAME
```

Run the `conditional_comp` example which demostrates how to check if
the failpoint library is disabled.

With failpoint enabled:

```bash
cargo run --example conditional_comp
```

Output:

```
failpoint! is enabled
```


With failpoint disabled:

```bash
cargo run --example conditional_comp --no-default-features
```

Output:

```
failpoint! is disabled
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
- Licensed under MIT OR Apache-2.0
