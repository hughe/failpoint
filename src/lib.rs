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

use std::fmt::Debug;

mod codepath_macros;
mod failpoint_macros;

#[cfg(feature = "failpoint_enabled")]
use std::sync::{LazyLock, Mutex, MutexGuard};

pub type Logger = Box<dyn Fn(String) + Send + Sync>;

#[cfg(feature = "failpoint_enabled")]
pub fn is_enabled() -> bool {
    true
}

#[cfg(not(feature = "failpoint_enabled"))]
pub fn is_enabled() -> bool {
    false
}

// HIDDEN DOC:
//
// Has to be public so that it can be accessed by the macro code
// from other crates, but it is not part of the public interface so we
// hide it from rust doc.
#[cfg(feature = "failpoint_enabled")]
#[derive(Debug, PartialEq)]
#[doc(hidden)]
pub enum Mode {
    Count,
    Trigger,
}

#[cfg(feature = "failpoint_enabled")]
#[doc(hidden)]
pub struct Inner {
    pub mode: Mode,

    pub counter: i64,

    logger: Option<Logger>,
    verbosity: i32,

    pub trigger: i64,
}

#[cfg(feature = "failpoint_enabled")]
impl Default for Inner {
    fn default() -> Self {
        Self {
            mode: Mode::Count,
            counter: 0,

            logger: None,
            verbosity: 1,

            trigger: i64::MAX,
        }
    }
}

#[cfg(feature = "failpoint_enabled")]
impl Inner {
    pub fn report_trigger(
        &mut self,
        crate_name: Option<&'static str>,
        file_name: &'static str,
        line_no: u32,
        desc: Option<&'static str>,
        error: &dyn Debug,
    ) {
        if self.verbosity >= 1 {
            if let Some(ref log) = self.logger {
                let loc = self.format_loc(crate_name, file_name, line_no);
                let msg = if let Some(d) = desc {
                    format!("Triggered failpoint \"{d}\" at {loc} returning {error:?}")
                } else {
                    format!("Triggered failpoint at {loc} returning {error:?}")
                };
                log(msg);
            }
        }
    }

    pub fn report_unexpected_failure(
        &mut self,
        crate_name: Option<&'static str>,
        file_name: &'static str,
        line_no: u32,
        desc: Option<&'static str>,
        error: &dyn Debug,
    ) {
        if self.verbosity >= 1 {
            if let Some(ref log) = self.logger {
                let loc = self.format_loc(crate_name, file_name, line_no);
                let msg = if let Some(d) = desc {
                    format!("Unexpected error in failpoint \"{d}\" at {loc} got {error:?}")
                } else {
                    format!("Unexpected error in failpoint at {loc} got {error:?}")
                };
                log(msg);
            }
        }
    }

    fn format_loc(
        &self,
        crate_name: Option<&'static str>,
        file_name: &'static str,
        line_no: u32,
    ) -> String {
        if let Some(c) = crate_name {
            format!("{file_name}:{line_no} in {c}")
        } else {
            format!("{file_name}:{line_no}")
        }
    }
}

#[cfg(feature = "failpoint_enabled")]
static STATE: LazyLock<State> = LazyLock::new(State::default);

// See HIDDEN DOC above.
#[cfg(feature = "failpoint_enabled")]
#[doc(hidden)]
pub struct State {
    pub mu: Mutex<Inner>,
}

#[cfg(feature = "failpoint_enabled")]
impl Default for State {
    fn default() -> Self {
        Self {
            mu: Mutex::new(Inner::default()),
        }
    }
}

// See HIDDEN DOC above.
#[cfg(feature = "failpoint_enabled")]
#[doc(hidden)]
pub fn get_state() -> &'static State {
    &STATE
}

// See HIDDEN DOC above.
#[cfg(feature = "failpoint_enabled")]
#[doc(hidden)]
pub fn lock_state<'a>() -> MutexGuard<'a, Inner> {
    let state = get_state();
    let g = state.mu.lock().unwrap();
    g
}

/// Enters count mode and resets the failpoint counter to zero.
///
/// In count mode, failpoints count how many times they are
/// encountered without triggering any errors. This is used for
/// discovering how many failpoints exist in a code path.
///
/// # Examples
///
/// ```rust
/// use failpoint::failpoint;
/// use anyhow::Error;
///
/// fn do_something() -> Result<(), Error> {
///     Ok(())
/// }
///
/// failpoint::start_counter();
/// let result = do_something();
/// let result = failpoint!(result, Error::msg("Test error"));
/// assert!(result.is_ok());
/// assert_eq!(failpoint::get_count(), 1);
/// ```
#[cfg(feature = "failpoint_enabled")]
pub fn start_counter() {
    let mut g = lock_state();
    g.mode = Mode::Count;
    g.counter = 0;
}

#[cfg(not(feature = "failpoint_enabled"))]
#[inline]
pub fn start_counter() {}

/// Enters trigger mode and sets which failpoint should trigger an error.
///
/// In trigger mode, the failpoint system will trigger an error at the specified
/// position in the sequence of failpoints encountered. The `trigger_after` parameter
/// specifies which failpoint in the sequence should trigger (1-indexed).
///
/// # Examples
///
/// ```rust
/// use failpoint::failpoint;
/// use anyhow::Error;
///
/// fn do_something() -> Result<(), Error> {
///     Ok(())
/// }
///
/// // Trigger the first failpoint encountered
/// failpoint::start_trigger(1);
/// let result = do_something();
/// let result = failpoint!(result, Error::msg("Test error"));
/// assert!(result.is_err());
/// ```
#[cfg(feature = "failpoint_enabled")]
pub fn start_trigger(trigger_after: i64) {
    let mut g = lock_state();
    g.mode = Mode::Trigger;
    g.trigger = trigger_after;
}

#[cfg(not(feature = "failpoint_enabled"))]
#[inline]
pub fn start_trigger(_trigger_after: i64) {}

/// Returns the current count of failpoints encountered in count mode.
///
/// This function returns the number of failpoints that have been encountered
/// since the last call to [`start_counter`]. Each failpoint macro call may
/// increment the counter by 1-3 depending on how many error cases it contains.
///
/// # Returns
///
/// The current failpoint counter value.
///
/// # Examples
///
/// ```rust
/// use failpoint::failpoint;
/// use anyhow::Error;
///
/// fn do_something() -> Result<(), Error> {
///     Ok(())
/// }
///
/// failpoint::start_counter();
///
/// let result = do_something();
/// let result = failpoint!(result, Error::msg("Error 1"));
/// let result = failpoint!(result, Error::msg("Error 2"));
/// assert_eq!(failpoint::get_count(), 2); // Two errors = count of 2
/// # _ = result;
/// ```
#[cfg(feature = "failpoint_enabled")]
pub fn get_count() -> i64 {
    let g = lock_state();
    g.counter
}

#[cfg(not(feature = "failpoint_enabled"))]
#[inline]
pub fn get_count() -> i64 {
    0
}

/// Sets the verbosity level for logging output.
///
/// Controls how much logging output is generated by the failpoint
/// system.  Higher values produce more verbose output. The argument
/// `v` is the verbosity level (0 = minimal, 1 = normal, 2 = verbose)
///
/// # Examples
///
/// ```rust
/// use failpoint;
///
/// failpoint::set_verbosity(2); // Enable verbose logging
/// failpoint::set_logger(Some(Box::new(|msg| println!("{}", msg))));
/// ```
#[cfg(feature = "failpoint_enabled")]
pub fn set_verbosity(v: i32) {
    let mut g = lock_state();
    g.verbosity = v;
}

#[cfg(not(feature = "failpoint_enabled"))]
#[inline]
pub fn set_verbosity(_v: i32) {}

/// Sets the logger function for failpoint output.
///
/// The logger function will be called with formatted log messages when
/// failpoints are triggered (in trigger mode) or when using the test_codepath
/// macro. Set to `None` to disable logging.
///
/// # Examples
///
/// ```rust
/// use failpoint;
///
/// // Enable logging to stdout
/// failpoint::set_verbosity(1);
/// failpoint::set_logger(Some(Box::new(|msg| println!("FAILPOINT: {}", msg))));
///
/// // Disable logging
/// failpoint::set_logger(None);
/// ```
#[cfg(feature = "failpoint_enabled")]
pub fn set_logger(l: Option<Logger>) {
    let mut g = lock_state();
    g.logger = l;
}

#[cfg(not(feature = "failpoint_enabled"))]
#[inline]
pub fn set_logger(_l: Option<Logger>) {}

// See HIDDEN DOC above.
#[cfg(feature = "failpoint_enabled")]
#[doc(hidden)]
pub fn log_if_verbose(level: i32, msg: String) {
    let g = lock_state();
    if g.verbosity >= level {
        if let Some(ref log_fn) = g.logger {
            log_fn(msg);
        }
    }
}

#[cfg(not(feature = "failpoint_enabled"))]
#[doc(hidden)]
#[inline]
pub fn log_if_verbose(_level: i32, _msg: String) {}

pub struct CodePathResult<T, E> {
    pub expected_trigger_count: i64,
    pub trigger_count: i64,
    pub unexpected_result: Option<Result<T, E>>,
}

impl<T, E> CodePathResult<T, E> {
    pub fn success(&self) -> bool {
        self.trigger_count == self.expected_trigger_count
    }
}
