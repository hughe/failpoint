use anyhow;

use failpoint::{failpoint, get_count, start_counter, start_trigger};

// This is the function that we're going to put a fail point
// around. It always returns success.  This might be a function that
// performs some IO, a system call or a library function.
//
// It's the thing that you want to simulate values for.
fn do_something_important() -> Result<(), anyhow::Error> {
    Ok(())
}


// This is how we use the failpoint macro to wrap a function.
fn do_something_else() -> Result<(), anyhow::Error> {
    // Some code ...

    // This says that if the failpoint is triggered, then we will
    // return an error with the message "Error 1". If the failpoint is
    // not triggered we will return whatever
    // `do_something_important()` returns.
    let res = failpoint!(do_something_important(), [ anyhow::Error::msg("Error 1")])?;

    // Some more code, do something with the result.

    _ = res;

    Ok(())
}

fn main() {
    // This is some code that will test `do_something_else()`.

    // Count the number of failpoints there are when we run
    // `do_something_else()`.
    start_counter();

    let res = do_something_else();

    assert!(res.is_ok());

    // There should be one fail point.
    assert_eq!(1, get_count());

    // Now run it in trigger mode.  The first time we run
    // `do_something_else()`, it will succeed. The second time, it
    // will fail with the error that we specified.
    start_trigger(2);

    // Succeed the first time, because the `trigger_after` parameter
    // was 2.
    let res = do_something_else();
    assert!(res.is_ok());

    // Fail the second time.
    let res = do_something_else();
    assert!(res.is_err());

    assert_eq!(format!("{}", res.err().unwrap()), "Error 1");
}
