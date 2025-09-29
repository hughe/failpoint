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

fn do_all_the_things()  -> Result<(), ExampleError> {

    // A failpoint that will run `do_the_first_thing()`.  If it is
    // triggered then it will return an `ExampleError::BadThing()`
    // error.
    failpoint!(do_the_first_thing(), [
	ExampleError::BadThing(io::Error::from(io::ErrorKind::NotFound))
    ])?;


    // A failpoint that will run `do_the_second_thing()`.  If it is
    // triggered then it will return an `ExampleError::WorseThing()`
    // error.
    failpoint!(do_the_second_thing(), [
	ExampleError::WorseThing("Oh no!".to_string())
    ])?;


    //Err(ExampleError::WorseThing("Test".to_string()))
    Ok(())
}

fn main() {

    // Find and excercise all the errors in `do_all_the_things()`.
    let res = test_codepath!(do_all_the_things());

    assert!(res.success());
}
