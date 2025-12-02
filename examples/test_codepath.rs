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
        ExampleError::BadThing(io::Error::from(io::ErrorKind::NotFound)),
        "do_the_first_thing BadThing"
    )?;

    let res = do_the_second_thing();
    // A failpoint that will run `do_the_second_thing()`.  If it is
    // triggered then it will return an `ExampleError::WorseThing()`
    // error.
    failpoint!(
        res,
        ExampleError::WorseThing("Oh no!".to_string()),
        "do_the_second_thing WorseThing"
    )?;

    Ok(())
}

fn main() {
    failpoint::set_logger(Some(Box::new(|m| println!("{}", m))));
    failpoint::set_verbosity(failpoint::Verbosity::Extreme);
    // Find and excercise all the errors in `do_all_the_things()`.
    let res = test_codepath!(do_all_the_things());

    // If we encounter an unexpected result, then this assert will
    // fail.
    assert!(res.success());

    res.report("test_codepath");
}
