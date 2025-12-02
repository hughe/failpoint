/// Integration tests for failpoint.
///
/// IMPORTANT: these tests must be run in a single thread, because
/// they use a global shared state.  For example:
///
/// ```
/// cargo test --all-targets --all-features -- --test-threads=1
/// ```
use anyhow::Error;
use std::io::Write;

use failpoint::{failpoint, test_codepath};
use test_log_collector::TestLogCollector;

// An important funtion whose result we want to change with a fail
// point in out tests.
fn important_function() -> Result<(), Error> {
    Ok(())
}

#[test]
fn test_counter_mode() {
    // A function to test.
    fn code_under_test() -> Result<(), Error> {
        let ret = important_function();

        let ret = failpoint!(ret, Error::msg("ERROR"), "Fail with \"ERROR\"");

        ret
    }

    // Start counter mode to count the failpoints.
    failpoint::start_counter();

    // We have not seen any failpoints.
    assert_eq!(0, failpoint::get_count());

    // Run the code under test.
    let res = code_under_test();

    // It should succeed.
    assert!(res.is_ok());

    // We have found 1 failpoint in the code under test.
    assert_eq!(1, failpoint::get_count());
}

#[test]
fn test_trigger_mode() {
    // A function to test.
    fn code_under_test() -> Result<(), Error> {
        let ret = important_function();
        let ret = failpoint!(ret, Error::msg("ERROR"), "Fail with \"ERROR\"");
        ret
    }

    // Run in trigger mode.  Trigger the first failpoint.
    failpoint::start_trigger(1);

    // Run the code under test.
    let res = code_under_test();

    // The test should have failed.
    assert!(res.is_err());

    // Assert that the message in res is "ERROR"
    assert_eq!(format!("{}", res.err().unwrap()), "ERROR");
}

#[test]
fn test_trigger_mode_two() {
    // A function to test.
    fn code_under_test() -> Result<(), Error> {
        let ret = important_function();
        let ret = failpoint!(ret, Error::msg("ERROR"), "Fail with \"ERROR\"");
        let ret = failpoint!(
            ret,
            Error::msg("ANOTHER ERROR"),
            "Fail with \"ANOTHER ERROR\""
        );

        ret
    }

    // Run in trigger mode.  Trigger the first failpoint.
    failpoint::start_trigger(2);

    // Run the code under test.
    let res = code_under_test();

    // The test should have failed.
    assert!(res.is_err());

    // Assert that the message in res is "ERROR"
    assert_eq!(format!("{}", res.err().unwrap()), "ANOTHER ERROR");
}

#[rustfmt::skip]
#[test]
fn test_test_codepath() {
    fn code_under_test() -> Result<(), Error> {
        let ret = important_function();
        let ret = failpoint!(ret, Error::msg("ERROR"), "Fail with \"ERROR\"");
        ret
    }

    let res = test_codepath! {
        codepath {
            code_under_test()
	}
    };

    assert!(res.success());

    assert_eq!(1, res.trigger_count);
    assert_eq!(1, res.expected_trigger_count);
    assert!(res.unexpected_result.is_none());
}

#[rustfmt::skip]
#[test]
fn test_test_codepath_two() {
    failpoint::set_verbosity(failpoint::Verbosity::Extreme);

    let log_collector = TestLogCollector::new_shared();
    let collector_clone = log_collector.clone();

    let logger = Box::new(move |msg: String| {
        let mut collector = collector_clone.lock().unwrap();
        writeln!(collector, "{}", msg).unwrap();
    });

    failpoint::set_logger(Some(logger));

    fn do_failpoint1() -> Result<(), Error> {
	let ret = important_function();
        let ret = failpoint!(ret, Error::msg("Error 1"), "Once");
	ret
    }

    fn do_failpoint2() -> Result<(), Error> {
	let ret = important_function();
        let ret = failpoint!(ret, Error::msg("Error 2"), "Twice");
	ret
    }

    fn code_under_test() -> Result<(), Error> {
        let res1 = do_failpoint1();
        if res1.is_err() { res1 } else { do_failpoint2() }
    }

    let res = test_codepath! {
	codepath {
	    code_under_test()
	}
    };

    assert!(res.success());

    assert_eq!(2, res.trigger_count);
    assert_eq!(2, res.expected_trigger_count);
    assert!(res.unexpected_result.is_none());

    // Check that log messages were written
    let collector = log_collector.lock().unwrap();
    let log_count = collector.count();
    assert!(log_count > 0, "Expected some log messages, but got none");

    let messages = collector.clone_lines();
    // You can also check for specific messages
    let has_trigger_msg = messages
        .iter()
        .any(|msg| msg.contains("Triggered Failpoint"));
    assert!(has_trigger_msg, "Expected a trigger message in logs");

    failpoint::set_verbosity(failpoint::Verbosity::None);
    failpoint::set_logger(None);
}

#[rustfmt::skip]
#[test]
fn test_test_codepath_before() {
    let mut before_ran = false;

    fn code_under_test() -> Result<(), Error> {
	let ret = important_function();
        let ret = failpoint!(ret, Error::msg("Error 1"), "Fail with \"ERROR 1\"");
	ret
    }

    let res = test_codepath! {
	before {
            before_ran = true;
	};
	codepath {
            code_under_test()
	};
	after {
	    // No after
	}
    };

    assert!(res.success());

    assert_eq!(1, res.trigger_count);
    assert_eq!(1, res.expected_trigger_count);
    assert!(res.unexpected_result.is_none());

    assert!(before_ran);
}

#[rustfmt::skip]
#[test]
fn test_test_codepath_after() {
    let mut after_ran = false;
    fn code_under_test() -> Result<(), Error> {
	let ret = important_function();
        let ret = failpoint!(ret, Error::msg("Error 1"), "Fail with \"ERROR 1\"");
	ret
    }

    let res = test_codepath! {
	codepath {
            code_under_test()
	};
	after{
            after_ran = true;
	}
    };

    assert!(res.success());

    assert_eq!(1, res.trigger_count);
    assert_eq!(1, res.expected_trigger_count);
    assert!(res.unexpected_result.is_none());

    assert!(after_ran);
}

#[rustfmt::skip]
#[test]
fn test_test_codepath_before_and_after() {
    let mut before_ran = false;
    let mut after_ran = false;

    fn code_under_test() -> Result<(), Error> {
        let ret = important_function();
        let ret = failpoint!(ret, Error::msg("Error 1"), "Fail with \"ERROR 1\"");
        ret
    }

    let res = test_codepath! {
	before {
            before_ran = true;
	};
	codepath {
            code_under_test()
	};
	after {
            after_ran = true;
	}
    };

    assert!(res.success());

    assert_eq!(1, res.trigger_count);
    assert_eq!(1, res.expected_trigger_count);
    assert!(res.unexpected_result.is_none());

    assert!(before_ran);
    assert!(after_ran);
}

#[rustfmt::skip]
#[test]
fn test_test_codepath_codepath_does_not_fail() {
    fn code_under_test() -> Result<(), Error> {
        let ret = important_function();
        _ = failpoint!(ret, Error::msg("Error 1"), "Fail with \"ERROR 1\"");
        Ok(())
    }

    let res = test_codepath! {
        codepath {
            code_under_test()
	}
    };

    assert!(!res.success());

    assert_eq!(0, res.trigger_count);
    assert_eq!(1, res.expected_trigger_count);
    assert!(res.unexpected_result.is_some());
}
