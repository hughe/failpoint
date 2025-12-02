use std::io;
use std::num::ParseIntError;

use thiserror::Error;

use failpoint::{failpoint, test_codepath};

// An error type.
#[derive(Error, Debug)]
enum MyError {
    #[error("Error reading file: {0}")]
    ReadError(#[from] io::Error),

    #[error("Parse error")]
    ParseError(#[from] ParseIntError),
}

// This function can return an IO error.
fn read_file() -> io::Result<()> {
    Ok(())
}

fn do_read_file() -> Result<(), MyError> {
    let res = read_file();

    // When this failpoint is triggered it will return an `io:Error`.
    let res = failpoint!(res, io::Error::from(io::ErrorKind::NotFound), "read_file");

    // Check the result and if we get an error convert it into a
    // `MyError`.
    match res {
        Err(e) => Err(MyError::ReadError(e)),
        Ok(()) => Ok(()),
    }
}

// This function can return a ParseIntError.
fn parse_file() -> Result<(), ParseIntError> {
    Ok(())
}

fn do_parse_file() -> Result<(), MyError> {
    let res = parse_file();
    // When triggered this failpoint will return a ParseIntError.  We
    // then convert that into a MyError.
    let res = failpoint!(
        res,
        "nope".parse::<i32>().err().unwrap(), // Makes a ParseIntError.
        "parse_file"
    );
    match res {
        Err(e) => Err(MyError::ParseError(e)),
        Ok(()) => Ok(()),
    }
}

fn load_file() -> Result<(), MyError> {
    do_read_file()?;

    do_parse_file()?;

    Ok(())
}

#[rustfmt::skip]
fn main() {
    // Find and excercise all the errors in `do_all_the_things()`.
    let res = test_codepath!{
	codepath {
	    load_file()
	}
    };

    assert!(res.success());
    assert_eq!(2, res.expected_trigger_count);
    assert_eq!(2, res.trigger_count);
    assert!(res.unexpected_result.is_none());
}
