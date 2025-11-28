use anyhow;

use failpoint::{failpoint, get_count, start_counter, start_trigger};

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
    start_counter();

    let res = do_something_else();

    // It succeeded because none of the failpoints were triggered.
    assert!(res.is_ok());

    // There should be one fail point.
    assert_eq!(1, get_count());

    // Now run it in trigger mode.  The first time we run
    // `do_something_else()`, it will fail, because the
    // `trigger_after` parameter to `start_trigger()` is `1`, which
    // means trigger the first failpoint.
    start_trigger(1);

    // Fail the first time because we t
    let res = do_something_else();
    assert!(res.is_err());

    assert_eq!(format!("{}", res.err().unwrap()), "Error 1");

    // The second time, it will succeed, because there are no more
    // failpoints to trigger.
    let res = do_something_else();
    assert!(res.is_ok());
}
