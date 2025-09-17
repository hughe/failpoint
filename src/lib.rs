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
enum Mode {
    Count,
    Trigger,
}

struct Inner {
    mode: Mode,

    counter: i64,

    reporter: Option<Reporter>,

    trigger: i64,
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

struct State {

    mu: Mutex<Inner>,

}

impl Default for State {
    fn default() -> Self {
        Self {
            mu: Mutex::new(Inner::default()),
        }
    }
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

pub const NO_DESC: &'static str = "";

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
		let state = &*STATE;
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
		let state = &*STATE;
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
		const CRATE_NAME: Option<&'static str> = core::option_env!("CARGO_CRATE_NAME");
		let state = &*STATE;
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

#[cfg(test)]
mod tests {

    use anyhow::Error;

    use super::*;

    #[test]
    fn test_counter_mode() {

	fn do_something() -> Result<(), Error> {
	    Ok(())
	}

	start_counter();

	assert_eq!(0, get_count());

	let res = failpoint!(do_something(), [ Error::msg("Error") ]);

	assert!(res.is_ok());

	assert_eq!(1, get_count());
    }


    #[test]
    fn test_counter_mode_two() {
	fn do_something() -> Result<(), Error> {
	    Ok(())
	}

	start_counter();

	let res = failpoint!(do_something(), [ Error::msg("Error 1"), Error::msg("Error 2") ]);

	assert!(res.is_ok());
	assert_eq!(2, get_count());
    }

    #[test]
    fn test_trigger_mode() {
	fn do_something() -> Result<(), Error> {
	    Ok(())
	}

	start_trigger(1);

	let res = failpoint!(do_something(), [ Error::msg("Error") ]);

	assert!(res.is_err());
    }

    #[test]
    fn test_trigger_mode_two() {
	fn do_something() -> Result<(), Error> {
	    Ok(())
	}

	fn do_failpoint() -> Result<(), Error> {
	    failpoint!(do_something(), [ Error::msg("Error 1"), Error::msg("Error 2") ])
	}

	start_trigger(3);

	let res0 = do_failpoint();
	assert!(res0.is_ok());

	let res1 = do_failpoint();
	assert!(res1.is_err());

	assert_eq!(format!("{}", res1.err().unwrap()), "Error 1");

	let res2 = do_failpoint();
	assert!(res2.is_err());

	assert_eq!(format!("{}", res2.err().unwrap()), "Error 2");
    }

}
