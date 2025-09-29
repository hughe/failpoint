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

use std::sync::{LazyLock, Mutex, MutexGuard};


pub type Logger = fn(msg: String);

// HIDDEN DOC:
//
// Has to be public so that it can be accessed by the macro code
// from other crates, but it is not part of the public interface so we
// hide it from rust doc.
#[derive(Debug, PartialEq)]
#[doc(hidden)]
pub enum Mode {
    Count,
    Trigger,
}

#[doc(hidden)]
pub struct Inner {
    pub mode: Mode,

    pub counter: i64,

    logger: Option<Logger>,
    verbosity: i32,

    pub trigger: i64,
}

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
            if let Some(log) = self.logger {
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

// See HIDDEN DOC above.
#[doc(hidden)]
pub struct State {
    pub mu: Mutex<Inner>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            mu: Mutex::new(Inner::default()),
        }
    }
}

// See HIDDEN DOC above.
#[doc(hidden)]
pub fn get_state() -> &'static State {
    &*STATE
}

pub fn lock_state<'a>() -> MutexGuard<'a, Inner> {
    let state = get_state();
    let g = state.mu.lock().unwrap();
    g
}

pub fn start_counter() {
    let mut g = lock_state();
    g.mode = Mode::Count;
    g.counter = 0;
}

pub fn start_trigger(trigger_after: i64) {
    let mut g = lock_state();
    g.mode = Mode::Trigger;
    g.trigger = trigger_after;
}

pub fn get_count() -> i64 {
    let g = lock_state();
    g.counter
}

pub fn set_verbosity(v: i32) {
    let mut g = lock_state();
    g.verbosity = v;
}


pub fn set_logger(l: Option<Logger>) {
    let mut g = lock_state();
    g.logger = l;
}

// See HIDDEN DOC above.
#[doc(hidden)]
pub fn get_logger_and_verbosity() -> (Option<Logger>, i32) {
    let g = lock_state();
    (g.logger, g.verbosity)
}

static STATE: LazyLock<State> = LazyLock::new(|| State::default());

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


#[macro_export]
macro_rules! test_codepath {

    (@log $level:expr, $msg:expr) => {
	{
	    use failpoint::get_logger_and_verbosity;
	    let (log_fn_opt, verbosity) = get_logger_and_verbosity();
	    if verbosity >= $level {
	    	if let Some(log_fn) = log_fn_opt {
	            log_fn($msg.to_string());
	     	}
	    }
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

	    test_codepath!(@log 1, format!("Triggered {trigger_count} of {error_count} errors"));

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
