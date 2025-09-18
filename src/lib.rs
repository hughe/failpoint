//! Failpoint is a error injection system.
//!
//! A failpoint is a place, where we can inject errors using the
//! [`failpoint!`] macro.  A failpoint looks something like this:
//!
//! ```rust
//! # use anyhow;
//! # fn main() -> Result<(), anyhow::Error> {
//! #   fn do_something() -> Result<(), Error> {
//! #     Ok(())
//! #   }
//! #     let res =
//! failpoint!(do_something(), [ anyhow::Error::msg("Error 1"), anyhow::Error::msg("Error 2") ])?;
//! #   assert_eq!(res.is_ok());
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
//! # use anyhow;
//! # fn main() -> Result<(), anyhow::Error> {
//! # fn do_something() -> Result<(), Error> {
//! #   Ok(())
//! # }
//! start_counter();
//!
//! let res = failpoint!(do_something(), [ anyhow::Error::msg("Error 1"), anyhow::Error::msg("Error 2") ])?;
//!
//! assert!(res.is_ok());
//! assert_eq!(2, get_count());
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
//! # use anyhow;
//! # fn main() -> Result<(), anyhow::Error> {
//! # fn do_something() -> Result<(), Error> {
//! #   Ok(())
//! # }
//! start_trigger(1);
//!
//! let res = failpoint!(do_something(), [ Error::msg("Error") ]);
//!
//! assert!(res.is_err());
//! # }
//! ```

use std::sync::{LazyLock, Mutex};


#[derive(Copy, Clone, Debug, Default)]
pub struct Location {
    crate_name: Option<&'static str>,
    file_name: &'static str,
    line_no: u32,

    desc: &'static str,
    err_no: usize,
}

pub type Reporter = fn(point: &Location);

#[derive(Debug, PartialEq)]
pub enum Mode {
    Count,
    Trigger,
}

pub struct Inner {
    pub mode: Mode,

    pub counter: i64,

    pub reporter: Option<Reporter>,

    pub trigger: i64,
    triggered_at: Option<Location>,

}

impl Default for Inner {
    fn default() -> Self {
        Self {
	    mode: Mode::Count,
            counter: 0,

            reporter: None, // Do we need this?

	    trigger: i64::MAX,
	    triggered_at: None,
        }
    }
}

impl Inner {

    pub fn _report(&mut self,
		   crate_name: Option<&'static str>,
		   file_name: &'static str,
		   line_no: u32,
		   desc: &'static str,
		   err_no: usize,
    ) {
	let point = Location {
	    crate_name,
	    file_name,
	    line_no,
	    desc,
	    err_no,
	};

	self.triggered_at = Some(point);

	if let Some(r) = self.reporter {
	    r(&point);
	}
    }

}

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

pub fn get_state() -> &'static State {
    &*STATE
}

pub fn start_counter() {
    let state = &*STATE;
    let mut g = state.mu.lock().unwrap();
    g.mode = Mode::Count;
    g.counter = 0;
}

pub fn start_trigger(trigger_after: i64) {
    let state = &*STATE;
    let mut g = state.mu.lock().unwrap();
    g.mode = Mode::Trigger;
    g.trigger = trigger_after;
    g.triggered_at = None;
}

pub fn get_count() -> i64 {
    let state = &*STATE;
    let g = state.mu.lock().unwrap();
    g.counter
}

pub fn get_triggered_at() -> Option<Location> {
    let state = &*STATE;
    let g = state.mu.lock().unwrap();
    g.triggered_at
}

static STATE: LazyLock<State> = LazyLock::new(|| State::default());

#[macro_export]
macro_rules! failpoint {

    ($exp: expr, [ $err1: expr, $err2: expr, $err3: expr ]) => {
	{
	    failpoint!($exp, "", [ $err1, $err2, $err3 ])
	}
    };

    ($exp: expr, $desc: expr, [ $err1: expr, $err2: expr, $err3: expr ]) => {
	{
	    // Evaluate exp OUTSIDE of the mutex to prevent possible
	    // deadlocks.
	    let res = $exp;

	    {
		use failpoint::{get_state, Mode};
		let state = get_state();
		let mut g = state.mu.lock().unwrap();
		const CRATE_NAME: Option<&'static str> = core::option_env!("CARGO_CRATE_NAME");

		if g.mode == Mode::Count {
		    g.counter = g.counter + 2;
		    res
		} else {
		    if g.trigger == 3 {
			g.trigger = g.trigger - 1;

			g._report(CRATE_NAME, file!(), line!(), $desc, 1);

			Err($err1)
		    } else {
			if g.trigger == 2 {
			    g.trigger = g.trigger - 1;

			    g._report(CRATE_NAME, file!(), line!(), $desc, 2);

			    Err($err2)
			} else {
			    if g.trigger == 1 {
				g.trigger = g.trigger - 1;
				g._report(CRATE_NAME, file!(), line!(), $desc, 3);

				Err($err3)
			    } else {
				g.trigger = g.trigger - 1;

				res
			    }
			}
		    }
		}
	    }
	}
    };


    ($exp: expr, [ $err1: expr, $err2: expr ]) => {
	{
	    failpoint!($exp, "", [ $err1, $err2 ])
	}
    };

    ($exp: expr, $desc: expr, [ $err1: expr, $err2: expr ]) => {
	{
	    // Evaluate exp OUTSIDE of the mutex to prevent possible
	    // deadlocks.
	    let res = $exp;

	    {
		use failpoint::{get_state, Mode};
		let state = get_state();
		let mut g = state.mu.lock().unwrap();
		const CRATE_NAME: Option<&'static str> = core::option_env!("CARGO_CRATE_NAME");

		if g.mode == Mode::Count {
		    g.counter = g.counter + 2;
		    res
		} else {
		    if g.trigger == 2 {
			g.trigger = g.trigger - 1;

			g._report(CRATE_NAME, file!(), line!(), $desc, 1);

			Err($err1)
		    } else {
			if g.trigger == 1 {
			    g.trigger = g.trigger - 1;
			    g._report(CRATE_NAME, file!(), line!(), $desc, 2);

			    Err($err2)
			} else {
			    g.trigger = g.trigger - 1;

			    res
			}
		    }
		}
	    }
	}
    };

    ($exp: expr, [ $err: expr ]) => {
	{
	    failpoint!($exp, "", [ $err ])
	}
    };

    ($exp: expr, $desc: expr, [ $err: expr ]) => {
	{
	    // Evaluate exp OUTSIDE of the mutex to prevent possible
	    // deadlocks.
	    let res = $exp;

	    {
		use failpoint::{get_state, Mode};
		const CRATE_NAME: Option<&'static str> = core::option_env!("CARGO_CRATE_NAME");
		let state = get_state();
		let mut g = state.mu.lock().unwrap();

		if g.mode == Mode::Count {
		    g.counter = g.counter + 1;
		    res
		} else {
		    g.trigger = g.trigger - 1;
		    if g.trigger == 0 {
			g._report(CRATE_NAME, file!(), line!(), $desc, 2);
			Err($err)
		    } else {
			res
		    }
		}
	    }
	}
    };
}
