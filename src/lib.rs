//! Failpoint is a error injection system.
//!
//! A failpoint is a place, where we can inject errors using the
//! [`failpoint!`] macro.  A failpoint looks something like this:
//!
//! ```rust
//! # use failpoint::failpoint;
//! # use anyhow;
//! # fn main() -> Result<(), anyhow::Error> {
//! #   fn do_something() -> Result<(), anyhow::Error> {
//! #     Ok(())
//! #   }
//! // Run the thing you want to fail.
//! let res = do_something();
//!
//! // Now simulate a failure.
//! let res = failpoint!(res, anyhow::Error::msg("Error 1"));
//!
//! // If the failpoint is triggered then res will be the error
//! // "Error 1".
//! # res
//! # }
//! ```
//!
//! The macro takes an identifier (`res`), which is the result of
//! calling a function that returns a `Result<T, E>`.  In this case
//! the function is `do_something()`. The macro takes a second
//! parameter which is an expression that evaluates to an error value
//! whose type must be `E`.  In this case the expression constructs an
//! `anyhow::Error`.
//!
//! The failpoint crate has two modes, "Count" mode and "Trigger"
//! mode.
//!
//! "Count" mode is used to count the number of failpoints in a code
//! path. When the crate is is "Count" mode and a failpoint is
//! encountered then the expression the value of the first parameter
//! will be returned by the macro and the count of failpoint errors in
//! the code path will be incremented.  "Count" mode is entered by
//! calling [`start_counter()`] before the code path is entered.  You
//! can find out how many failpoint errors there are on the code path
//! by calling [`get_count()`] after the codepath has run in "Count"
//! mode.
//!
//! ```rust
//! # use failpoint::failpoint;
//! # use anyhow;
//! # fn main() -> Result<(), anyhow::Error> {
//! # fn do_something() -> Result<(), anyhow::Error> {
//! #   Ok(())
//! # }
//! failpoint::start_counter();
//!
//! let res = do_something();
//!
//! let res = failpoint!(res, anyhow::Error::msg("Error 1"));
//!
//! // We don't inject any errors in "Count" mode, so the result will
//! // Ok().
//! assert!(res.is_ok());
//!
//! // There is one failpoint in code path.
//! assert_eq!(1, failpoint::get_count());
//! # Ok(())
//! # }
//! ```
//!
//! In "Trigger" mode the failpoint containing the nth error on the
//! code path will be triggered.  When the failpoint is triggered it
//! will return the value of the error expression.
//!
//! "Trigger" mode is entered by calling `start_trigger(n)` which will
//! cause a failpoint to return the nth error on the codepath.
//!
//! ```rust
//! # use failpoint::failpoint;
//! # use anyhow;
//! # fn main() {
//! # fn do_something() -> Result<(), anyhow::Error> {
//! #   Ok(())
//! # }
//! failpoint::start_trigger(1);
//!
//! let res = do_something();
//! let res = failpoint!(res, anyhow::Error::msg("Error"));
//!
//! assert!(res.is_err());
//! # }
//! ```

mod codepath_macros;
mod codepath_state;
mod failpoint_macros;
mod failpoint_state;

// Re-export public API from failpoint_state
pub use failpoint_state::{
    get_count, is_enabled, set_logger, set_verbosity, start_counter, start_trigger, Location,
    Logger, Verbosity,
};

#[cfg(feature = "failpoint_enabled")]
pub use failpoint_state::{get_state, lock_state, log_if_verbose, Inner, Mode, State};

pub use codepath_state::CodePathResult;
