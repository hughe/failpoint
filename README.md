# failpoint - Fault Injection

A lightweight Rust library for fault injection testing. Simulate errors in your code to verify error handling paths work correctly.

## Features

- **Simple failpoint injection** - Add failpoints to your code that can simulate errors
- **Automatic error path testing** - Use `test_codepath!` to systematically test all error conditions
- **Zero runtime overhead** - Failpoints only activate during testing

## Usage

### Basic Failpoint Injection

Use the `failpoint!` macro to inject potential failure points in your code:

```rust
use failpoint::{failpoint, start_trigger};

fn database_operation() -> Result<String, String> {
    let result = failpoint!(Ok("success".to_string()), [
        "Connection failed".to_string(),
        "Timeout error".to_string(),
        "Permission denied".to_string()
    ]);
    result
}

// In tests, trigger specific errors
start_trigger(1); // Triggers the first error
let result = database_operation();
assert!(result.is_err());
```

### Automated Error Path Testing

Use `test_codepath!` to automatically test all error paths:

```rust
use failpoint::{failpoint, test_codepath};

fn process_data() -> Result<i32, String> {
    let value: Result<i32, String> = failpoint!(Ok(42), [
        "Error 1".to_string(),
        "Error 2".to_string(),
        "Error 3".to_string()
    ]);
    value
}

#[test]
fn test_all_errors() {
    let result = test_codepath!({
        // Setup code runs before each iteration
    };
    {
        process_data()
    };
    {
        // Cleanup code runs after each iteration
    });

    assert!(result.success());
}
```

## Building and Testing

### Build the library

```bash
cargo build
```

### Run tests

Because the library uses a thread-safe singleton for holding its state, tests must run sequentially:

```bash
cargo test -- --test-threads=1
```

### Run documentation tests

```bash
cargo test --doc -- --test-threads=1
```

### Generate documentation

```bash
cargo doc --open
```



## Other work

Failpoint is inspired by
[fault-injection](https://crates.io/crates/fault-injection).  It
differs from `fault-injection` in the following ways:

1. `failpoint` can inject different types of error. `fault-injection`
   is limited to injecting `std::io::Error` only.
2. `failpoint` is much heavier weight, it uses `Mutex`s and
   `LazyLock`s where `fault-injection` uses a single atomic integer
   ...
3. `fault-injection` is simpler.





