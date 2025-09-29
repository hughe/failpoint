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
//! failpoint!(do_something(), [ anyhow::Error::msg("Error 1"), anyhow::Error::msg("Error 2") ])?;
//! # Ok(())
//! # }
//! ```
//!
//! The macro wraps an expression (parameter `$exp`), which is usually
//! a function call.  In this case the expression is `do_something()`.
//! The expression must evaluate to a `Result<T, E>`. The macro also
//! takes one to three parameters (`$err1`, `$err2`, ...)  in square
//! brackets which are expressions that evaluate to error values whose
//! type must be `E`.  In this case there are two expressions, both of
//! which constructs an `anyhow::Error`.
//!
//! The failpoint crate has two modes, "Count" mode and "Trigger"
//! mode.
//!
//! "Count" mode is used to count the number of failpoints in a code
//! path. When the crate is is "Count" mode and a fail point is
//! encountered then the expression (`$exp`) will be evaluated and
//! returned by the macro and the count of failpoint errors in the
//! code path will be incremented.  "Count" mode is entered by calling
//! [`start_counter()`] before the code path is entered.  You can find out
//! how many failpoint errors there are on the code path by calling
//! [`get_count()`] after the codepath has run in "Count" mode.
//!
//! ```rust
//! # use failpoint::{failpoint, start_counter, get_count};
//! # use anyhow;
//! # fn main() -> Result<(), anyhow::Error> {
//! # fn do_something() -> Result<(), anyhow::Error> {
//! #   Ok(())
//! # }
//! start_counter();
//!
//! failpoint!(do_something(), [ anyhow::Error::msg("Error 1"), anyhow::Error::msg("Error 2") ])?;
//!
//! assert_eq!(2, get_count());
//! # Ok(())
//! # }
//! ```
//!
//! In "Trigger" mode the failpoint containing the nth error on the code path will be
//! triggered.  When the failpoint is triggered, it will firstly
//! evaluate the expression `$exp` and then it will return the value
//! of one of the error expressions.  The error values are returned in
//! the order listed, the first time the failpoint is triggered
//! `$err1` is returned, the second time `$err2`, ...
//!
//! "Trigger" mode is entered by calling `start_trigger(n)` which will
//! cause a failpoint to return the nth error on the codepath.
//!
//! ```rust
//! # use failpoint::{failpoint, start_trigger};
//! # use anyhow;
//! # fn main() {
//! # fn do_something() -> Result<(), anyhow::Error> {
//! #   Ok(())
//! # }
//! start_trigger(1);
//!
//! let res = failpoint!(do_something(), [ anyhow::Error::msg("Error") ]);
//!
//! assert!(res.is_err());
//! # }
//! ```

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

            logger: None, // Do we need this?
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
        err_no: usize,
    ) {
        if self.verbosity >= 1 {
            if let Some(ref log) = self.logger {
                let loc = if let Some(c) = crate_name {
                    format!("{file_name}:{line_no} error {err_no} in {c}")
                } else {
                    format!("{file_name}:{line_no} error {err_no}")
                };
                let msg = if let Some(d) = desc {
                    format!("Triggered failpoint \"{d}\" at {loc}")
                } else {
                    format!("Triggered failpoint at {loc}")
                };
                log(msg);
            }
        }
    }
}

#[cfg(feature = "failpoint_enabled")]
static STATE: LazyLock<State> = LazyLock::new(|| State::default());

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
    &*STATE
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
/// In count mode, failpoints count how many times they are encountered without
/// triggering any errors. This is useful for discovering how many failpoints
/// exist in a code path.
///
/// # Examples
///
/// ```rust
/// use failpoint::{failpoint, start_counter, get_count};
/// use anyhow::Error;
///
/// fn do_something() -> Result<(), Error> {
///     Ok(())
/// }
///
/// start_counter();
/// let result = failpoint!(do_something(), [Error::msg("Test error")]);
/// assert!(result.is_ok());
/// assert_eq!(get_count(), 1);
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
/// use failpoint::{failpoint, start_trigger};
/// use anyhow::Error;
///
/// fn do_something() -> Result<(), Error> {
///     Ok(())
/// }
///
/// // Trigger the first failpoint encountered
/// start_trigger(1);
/// let result = failpoint!(do_something(), [Error::msg("Test error")]);
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
/// use failpoint::{failpoint, start_counter, get_count};
/// use anyhow::Error;
///
/// fn do_something() -> Result<(), Error> {
///     Ok(())
/// }
///
/// start_counter();
/// let _result = failpoint!(do_something(), [Error::msg("Error 1"), Error::msg("Error 2")]);
/// assert_eq!(get_count(), 2); // Two errors = count of 2
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
/// use failpoint::{set_verbosity, set_logger};
///
/// set_verbosity(2); // Enable verbose logging
/// set_logger(Some(Box::new(|msg| println!("{}", msg))));
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
/// use failpoint::{set_logger, set_verbosity};
///
/// // Enable logging to stdout
/// set_verbosity(1);
/// set_logger(Some(Box::new(|msg| println!("FAILPOINT: {}", msg))));
///
/// // Disable logging
/// set_logger(None);
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

/// Injects a failpoint into code for testing error conditions.
///
/// The `failpoint!` macro wraps an expression (usually a function call) that returns a `Result<T, E>`
/// and provides a mechanism to inject errors during testing. The macro takes the expression and
/// one to three error values that can be returned instead of the original result.
///
/// # Arguments
///
/// * `$exp` - An expression that evaluates to a `Result<T, E>`
/// * `$err1`, `$err2`, `$err3` - Error expressions of type `E` (1-3 errors supported)
/// * `$desc` - Optional description string for logging (when provided)
///
/// # Modes
///
/// The failpoint system operates in two modes:
///
/// - **Count mode**: Counts how many failpoints exist in a code path without triggering errors
/// - **Trigger mode**: Triggers specific errors at specific failpoints
///
/// # Examples
///
/// ## Basic usage with single error
///
/// ```rust
/// use failpoint::{failpoint, start_counter, get_count, start_trigger};
/// use anyhow::Error;
///
/// fn do_something() -> Result<(), Error> {
///     Ok(())
/// }
///
/// // Count mode: count failpoints without triggering
/// start_counter();
/// let result = failpoint!(do_something(), [Error::msg("Test error")]);
/// assert!(result.is_ok());
/// assert_eq!(get_count(), 1);
///
/// // Trigger mode: trigger the first error
/// start_trigger(1);
/// let result = failpoint!(do_something(), [Error::msg("Test error")]);
/// assert!(result.is_err());
/// ```
///
/// ## Multiple errors
///
/// ```rust
/// use failpoint::{failpoint, start_trigger};
/// use anyhow::Error;
///
/// fn do_something() -> Result<(), Error> {
///     Ok(())
/// }
///
/// // Trigger the second error in the sequence
/// start_trigger(2);
/// let result = failpoint!(do_something(), [
///     Error::msg("Error 1"),
///     Error::msg("Error 2")
/// ]);
/// assert!(result.is_err());
/// assert_eq!(result.unwrap_err().to_string(), "Error 1");
/// ```
///
/// ## With description for logging
///
/// ```rust
/// use failpoint::{failpoint, start_trigger, set_logger, set_verbosity};
/// use anyhow::Error;
///
/// fn do_something() -> Result<(), Error> {
///     Ok(())
/// }
///
/// set_verbosity(1);
/// set_logger(Some(Box::new(|msg| println!("{}", msg))));
///
/// start_trigger(1);
/// let result = failpoint!(do_something(), "Database connection", [
///     Error::msg("Connection failed")
/// ]);
/// assert!(result.is_err());
/// ```
#[cfg(feature = "failpoint_enabled")]
#[macro_export]
macro_rules! failpoint {
    ($exp: expr, [ $err1: expr, $err2: expr, $err3: expr ]) => {{
        failpoint!(@i3 $exp, None, [$err1, $err2, $err3])
    }};

    ($exp: expr, $desc: expr, [ $err1: expr, $err2: expr, $err3: expr ]) => {{
	failpoint!(@i3 $exp, Some($desc), [$err1, $err2, $err3])
    }};

    (@i3 $exp: expr, $desc: expr, [ $err1: expr, $err2: expr, $err3: expr ]) => {{
        // Evaluate exp OUTSIDE of the mutex to prevent possible
        // deadlocks.
        let res = $exp;

        {
            use failpoint::{lock_state, Mode};
            let mut g = lock_state();
            const CRATE_NAME: Option<&'static str> = core::option_env!("CARGO_CRATE_NAME");

            if g.mode == Mode::Count {
                g.counter = g.counter + 2;
                res
            } else {
                if g.trigger == 3 {
                    g.trigger = g.trigger - 1;

                    g.report_trigger(CRATE_NAME, file!(), line!(), $desc, 1);

                    Err($err1)
                } else {
                    if g.trigger == 2 {
                        g.trigger = g.trigger - 1;

                        g.report_trigger(CRATE_NAME, file!(), line!(), $desc, 2);

                        Err($err2)
                    } else {
                        if g.trigger == 1 {
                            g.trigger = g.trigger - 1;

			    g.report_trigger(CRATE_NAME, file!(), line!(), $desc, 3);

                            Err($err3)
                        } else {
                            g.trigger = g.trigger - 1;

                            res
                        }
                    }
                }
            }
        }
    }};

    ($exp: expr, [ $err1: expr, $err2: expr ]) => {{
        failpoint!(@i2 $exp, None, [$err1, $err2])
    }};

    ($exp: expr, $desc: expr, [ $err1: expr, $err2: expr ]) => {{
        failpoint!(@i2 $exp, Some($desc), [$err1, $err2])
    }};

    (@i2 $exp: expr, $desc: expr, [ $err1: expr, $err2: expr ]) => {{
        // Evaluate exp OUTSIDE of the mutex to prevent possible
        // deadlocks.
        let res = $exp;

        {
            use failpoint::{lock_state, Mode};
            let mut g = lock_state();
            const CRATE_NAME: Option<&'static str> = core::option_env!("CARGO_CRATE_NAME");

            if g.mode == Mode::Count {
                g.counter = g.counter + 2;
                res
            } else {
                if g.trigger == 2 {
                    g.trigger = g.trigger - 1;

                    g.report_trigger(CRATE_NAME, file!(), line!(), $desc, 1);

                    Err($err1)
                } else {
                    if g.trigger == 1 {
                        g.trigger = g.trigger - 1;

			g.report_trigger(CRATE_NAME, file!(), line!(), $desc, 2);

                        Err($err2)
                    } else {
                        g.trigger = g.trigger - 1;

                        res
                    }
                }
            }
        }
    }};

    ($exp: expr, [ $err: expr ]) => {{
        failpoint!(@i1 $exp, None, [$err])
    }};

    ($exp: expr, $desc: expr, [ $err: expr ]) => {{
        failpoint!(@i1 $exp, Some($desc), [$err])
    }};

    (@i1 $exp: expr, $desc: expr, [ $err: expr ]) => {{
        // Evaluate exp OUTSIDE of the mutex to prevent possible
        // deadlocks.
        let res = $exp;

        {
            use failpoint::{lock_state, Mode};
            const CRATE_NAME: Option<&'static str> = core::option_env!("CARGO_CRATE_NAME");
            let mut g = lock_state();

            if g.mode == Mode::Count {
                g.counter = g.counter + 1;
                res
            } else {
                g.trigger = g.trigger - 1;
                if g.trigger == 0 {
                    g.report_trigger(CRATE_NAME, file!(), line!(), $desc, 2);
                    Err($err)
                } else {
                    res
                }
            }
        }
    }};
}

#[cfg(not(feature = "failpoint_enabled"))]
#[macro_export]
macro_rules! failpoint {
    ($exp: expr, [ $err1: expr, $err2: expr, $err3: expr ]) => {{
        let _ = (|| $err1);
        let _ = (|| $err2);
        let _ = (|| $err3);
        $exp
    }};

    ($exp: expr, $desc: expr, [ $err1: expr, $err2: expr, $err3: expr ]) => {{
        let _ = $desc;
        let _ = (|| $err1);
        let _ = (|| $err2);
        let _ = (|| $err3);
        $exp
    }};

    ($exp: expr, [ $err1: expr, $err2: expr ]) => {{
        let _ = (|| $err1);
        let _ = (|| $err2);
        $exp
    }};

    ($exp: expr, $desc: expr, [ $err1: expr, $err2: expr ]) => {{
        let _ = $desc;
        let _ = (|| $err1);
        let _ = (|| $err2);
        $exp
    }};

    ($exp: expr, [ $err: expr ]) => {{
        let _ = (|| $err);
        $exp
    }};

    ($exp: expr, $desc: expr, [ $err: expr ]) => {{
        let _ = $desc;
        let _ = (|| $err);
        $exp
    }};
}

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


/// Tests a code path by triggering all possible failpoints.
///
/// This macro runs the provided code path twice: first in COUNT mode to discover
/// how many failpoints exist, then in TRIGGER mode to systematically trigger each
/// failpoint and verify error handling. Setup and cleanup blocks can be provided
/// to reset state between iterations.
///
/// # Syntax
///
/// ```ignore
/// test_codepath!({ setup }; code_path; { cleanup })
/// test_codepath!(code_path; { cleanup })
/// test_codepath!(code_path)
/// ```
///
/// # Returns
///
/// Returns a [`CodePathResult`] that indicates whether all failpoints were successfully
/// triggered. Call `.success()` to check if the test passed.
///
/// # Example
///
/// ```
/// use failpoint::{failpoint, test_codepath};
///
/// fn process_data() -> Result<i32, String> {
///     let value: Result<i32, String> = failpoint!(Ok(42), [
///         "Simulated error 1".to_string(),
///         "Simulated error 2".to_string(),
///         "Simulated error 3".to_string()
///     ]);
///     value
/// }
///
/// let result = test_codepath!({
///     // Setup: runs before each iteration
/// };
/// {
///     // Code path to test
///     process_data()
/// };
/// {
///     // Cleanup: runs after each iteration
/// });
///
/// assert!(result.success());
/// ```
#[cfg(feature = "failpoint_enabled")]
#[macro_export]
macro_rules! test_codepath {

    (@log $level:expr, $msg:expr) => {
	{
	    use failpoint::log_if_verbose;
	    log_if_verbose($level, $msg.to_string());
	}
    };

    { $before: block ; $codepath: expr ; $after: block } => {
	{
	    use failpoint::{start_counter, start_trigger, Mode, get_count, CodePathResult};
	    let mut mode = Mode::Count;
	    let mut trigger_count = 0;
	    let mut error_count = i64::MAX;

	    let unexpected_result = loop {
		if mode == Mode::Trigger && trigger_count > error_count  {
		    break None;
		}

		test_codepath!(@log 2, "\n------------------------------------------------------------".to_string());
		test_codepath!(@log 2,
			       format!("Testing codepath in {} mode", if mode == Mode::Count { "COUNT" } else { "TRIGGER" }));

		test_codepath!(@log 2, "Running before block".to_string());

		$before;

		if mode == Mode::Count {
		    start_counter();
		    test_codepath!(@log 2, "Running codepath in COUNT mode".to_string());
		} else {
		    start_trigger(trigger_count);
		    test_codepath!(@log 2, format!("Running codepath in TRIGGER mode, will trigger error {}", trigger_count));
		}

		let res = $codepath;

		if mode == Mode::Count {
		    if res.is_err() {
			test_codepath!(@log 0,
				       "Error returned by codepath in count mode. Expected codepath to succeed.".to_string());
			break Some(res)
		    }
		} else {
		    if !res.is_err() {
			test_codepath!(@log 0,
				       format!("Codepath did not fail in trigger mode for error {}.  Expected codepath to fail.",
					       trigger_count));
			break Some(res)
		    }
		}

		if mode == Mode::Count {
		    mode = Mode::Trigger;
		    trigger_count = 1;
		    error_count = get_count();
		} else {
		    trigger_count += 1;
		}

		test_codepath!(@log 1, "Running after block");

		$after;
	    };

	    test_codepath!(@log 1, format!("Triggered {} of {} errors", trigger_count - 1, error_count));

	    let ret = CodePathResult{
		expected_trigger_count: error_count,
		trigger_count: trigger_count - 1,
		unexpected_result,
	    };

	    ret
	}
    };

    { $codepath: expr ; $after: block } => {
	test_codepath!{ {}; $codepath; $after }
    };

    { $codepath: expr } => {
	test_codepath!{ {}; $codepath; {} }
    };

}

#[cfg(not(feature = "failpoint_enabled"))]
#[macro_export]
macro_rules! test_codepath {
    { $before: block ; $codepath: expr ; $after: block } => {{
        use failpoint::CodePathResult;
        $before;
        let res = $codepath;
        $after;
        CodePathResult::<_, _> {
            expected_trigger_count: 0,
            trigger_count: 0,
            unexpected_result: Some(res),
        }
    }};

    { $codepath: expr ; $after: block } => {
        test_codepath!{ {}; $codepath; $after }
    };

    { $codepath: expr } => {
        test_codepath!{ {}; $codepath; {} }
    };
}
