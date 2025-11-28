# failpoint - Fault Injection

A lightweight Rust library for fault injection testing. Simulate
errors in your code to verify error handling paths work correctly.

## Features

- **Simple failpoint injection** - Add failpoints to your code that can simulate errors
- **Automatic error path testing** - Use `test_codepath!` to systematically test all error conditions

## Usage

### Basic Failpoint Injection

Use the `failpoint!` macro to inject potential failure points in your code:

```rust
use anyhow;

use failpoint::failpoint;

// This is the function that we're going to put a fail point
// around. This might be a function that performs some IO, a system
// call or a library function.  It's the thing that you want to
// simulate errors for.
fn do_something_important() -> Result<(), anyhow::Error> {
    Ok(())
}

// This is how we use the failpoint macro to simulate errors in
// `do_something_important()`.
fn do_something_else() -> Result<(), anyhow::Error> {
    // Some code ...

    let res = do_something_important();
    // This says that if the failpoint is triggered, then we will
    // return an error with the message "Error 1". If the failpoint is
    // not triggered we will return whatever
    // `do_something_important()` returns.
    let res = failpoint!(res, anyhow::Error::msg("Error 1"));

    res
}

fn main() {
    // This is some code that will test `do_something_else()`.

    // First we count the number of failpoints there are when we run
    // `do_something_else()`.
    failpoint::start_counter();

    let res = do_something_else();

    // It succeeded because none of the failpoints were triggered.
    assert!(res.is_ok());

    // There should be one fail point.
    assert_eq!(1, failpoint::get_count());

    // Now run it in trigger mode.  The first time we run
    // `do_something_else()`, it will fail, because the
    // `trigger_after` parameter to `start_trigger()` is `1`, which
    // means trigger the first failpoint.
    failpoint::start_trigger(1);

    // Fail the first time because we t
    let res = do_something_else();
    assert!(res.is_err());

    assert_eq!(format!("{}", res.err().unwrap()), "Error 1");

    // The second time, it will succeed, because there are no more
    // failpoints to trigger.
    let res = do_something_else();
    assert!(res.is_ok());
}
```

### Automated Error Path Testing

Use `test_codepath!` to automatically test all error paths:

```rust
use std::io;

use thiserror::Error;

use failpoint::{failpoint, test_codepath};

// An error type.
#[derive(Error, Debug)]
enum ExampleError {
    #[error("a bad thing happened")]
    BadThing(#[from] io::Error),

    #[error("a worse thing happened")]
    WorseThing(String),
}

fn do_the_first_thing() -> Result<(), ExampleError> {
    Ok(())
}

fn do_the_second_thing() -> Result<(), ExampleError> {
    Ok(())
}

fn do_all_the_things() -> Result<(), ExampleError> {
    let res = do_the_first_thing();

    // A failpoint that will run `do_the_first_thing()`.  If it is
    // triggered then it will return an `ExampleError::BadThing()`
    // error.
    failpoint!(
        res,
        ExampleError::BadThing(io::Error::from(io::ErrorKind::NotFound))
    )?;

    let res = do_the_second_thing();
    // A failpoint that will run `do_the_second_thing()`.  If it is
    // triggered then it will return an `ExampleError::WorseThing()`
    // error.
    failpoint!(res, ExampleError::WorseThing("Oh no!".to_string()))?;

    Ok(())
}

fn main() {
    // Find and excercise all the errors in `do_all_the_things()`.
    let res = test_codepath!(do_all_the_things());

    // If we encounter an unexpected result, then this assert will
    // fail.
    assert!(res.success());
}
```

## Compiling Out Failpoints

By default, the `failpoint` library is fully enabled via the `failpoint_enabled` feature flag. For production builds, you can compile out all failpoint functionality to achieve zero runtime overhead.

### Disabling failpoints

Add the library to your `Cargo.toml` with `default-features = false`:

```toml
[dependencies]
failpoint = { version = "2.0", default-features = false }
```

When disabled, all failpoint macros and functions become no-ops that should be optimized away by the compiler, resulting in zero runtime cost.


### Explicitly controlling the feature

You can also explicitly enable or disable the feature:

```toml
# Explicitly enable
[dependencies]
failpoint = { version = "2.0", features = ["failpoint_enabled"] }

# Explicitly disable
[dependencies]
failpoint = { version = "2.0", default-features = false }
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

### Examples

Run an example.  See the `examples` directory

```bash
cargo run --example EXAMPLE-NAME
```

E.g., 

```bash
cargo run --example failpoint
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
failpoint! is enabled
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

## Thanks

Thank you to the authors of `fault-injection` for showing me how to
get started.



