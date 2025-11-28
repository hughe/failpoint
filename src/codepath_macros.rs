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
/// ```rust
/// use failpoint::{failpoint, test_codepath};
///
/// fn process_data() -> Result<i32, String> {
///     let value: Result<i32, String> = Ok(42);
///     let value = failpoint!(value, "Simulated error 1".to_string());
///     let value = failpoint!(value, "Simulated error 2".to_string());
///     let value = failpoint!(value, "Simulated error 3".to_string());
///     value
/// }
///
/// let result = test_codepath!(
///     {
///         // Setup: runs before each iteration
///     };
///     {
///         // Code path to test
///         process_data()
///     };
///     {
///         // Cleanup: runs after each iteration
///     }
/// );
///
/// assert!(result.success());
///
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
