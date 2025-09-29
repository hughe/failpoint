use anyhow::Error;
use std::io::Write;

use failpoint::*;
use test_log_collector::TestLogCollector;


#[test]
fn test_counter_mode() {
    fn do_something() -> Result<(), Error> {
        Ok(())
    }

    start_counter();

    assert_eq!(0, get_count());

    let res = failpoint!(do_something(), [Error::msg("Error")]);

    assert!(res.is_ok());

    assert_eq!(1, get_count());
}

#[test]
fn test_counter_mode_two() {
    fn do_something() -> Result<(), Error> {
        Ok(())
    }

    start_counter();

    let res = failpoint!(
        do_something(),
        [Error::msg("Error 1"), Error::msg("Error 2")]
    );

    assert!(res.is_ok());
    assert_eq!(2, get_count());
}

#[test]
fn test_trigger_mode() {
    fn do_something() -> Result<(), Error> {
        Ok(())
    }

    start_trigger(1);

    let res = failpoint!(do_something(), [Error::msg("Error")]);

    assert!(res.is_err());
}

#[test]
fn test_trigger_mode_two() {
    fn do_something() -> Result<(), Error> {
        Ok(())
    }

    fn do_failpoint() -> Result<(), Error> {
        failpoint!(
            do_something(),
            [Error::msg("Error 1"), Error::msg("Error 2")]
        )
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

#[test]
fn test_test_codepath() {
    fn do_something() -> Result<(), Error> {
        Ok(())
    }

    fn do_failpoint() -> Result<(), Error> {
        failpoint!(do_something(), [Error::msg("Error 1")])
    }

    let res = test_codepath! {
            do_failpoint()
    };

    assert!(res.success());

    assert_eq!(1, res.trigger_count);
    assert_eq!(1, res.expected_trigger_count);
    assert!(res.unexpected_result.is_none());
}


#[test]
fn test_test_codepath_two() {

    set_verbosity(2);

    let log_collector = TestLogCollector::new_shared();
    let collector_clone = log_collector.clone();

    let logger = Box::new(move |msg: String| {
        let mut collector = collector_clone.lock().unwrap();
        writeln!(collector, "{}", msg).unwrap();
    });

    set_logger(Some(logger));

    fn do_something() -> Result<(), Error> {
        Ok(())
    }

    fn do_failpoint1() -> Result<(), Error> {
        failpoint!(do_something(), "Once", [Error::msg("Error 1")])
    }

    fn do_failpoint2() -> Result<(), Error> {
        failpoint!(do_something(), "Twice", [Error::msg("Error 2")])
    }

    let res = test_codepath! {
	{
	    let res1 = do_failpoint1();
	    if res1.is_err() {
		res1
	    } else {
		do_failpoint2()
	    }
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
    let has_trigger_msg = messages.iter().any(|msg| msg.contains("Triggered failpoint"));
    assert!(has_trigger_msg, "Expected a trigger message in logs");

    set_verbosity(0);
    set_logger(None);
}

#[test]
fn test_test_codepath_before() {
    let mut before_ran = false;
    fn do_something() -> Result<(), Error> {
        Ok(())
    }

    fn do_failpoint() -> Result<(), Error> {
        failpoint!(do_something(), [Error::msg("Error 1")])
    }

    let res = test_codepath! {
	{
	    before_ran = true;
	};
	{
            do_failpoint()
	};
	{
	}
    };

    assert!(res.success());

    assert_eq!(1, res.trigger_count);
    assert_eq!(1, res.expected_trigger_count);
    assert!(res.unexpected_result.is_none());

    assert!(before_ran);
}

#[test]
fn test_test_codepath_after() {
    let mut after_ran = false;
    fn do_something() -> Result<(), Error> {
        Ok(())
    }

    fn do_failpoint() -> Result<(), Error> {
        failpoint!(do_something(), [Error::msg("Error 1")])
    }

    let res = test_codepath! {
	{
            do_failpoint()
	};
	{
	    after_ran = true;
	}
    };

    assert!(res.success());

    assert_eq!(1, res.trigger_count);
    assert_eq!(1, res.expected_trigger_count);
    assert!(res.unexpected_result.is_none());

    assert!(after_ran);
}

#[test]
fn test_test_codepath_before_and_after() {
    let mut before_ran = false;
    let mut after_ran = false;

    fn do_something() -> Result<(), Error> {
        Ok(())
    }

    fn do_failpoint() -> Result<(), Error> {
        failpoint!(do_something(), [Error::msg("Error 1")])
    }

    let res = test_codepath! {
	{
	    before_ran = true;
	};
	{
            do_failpoint()
	};
	{
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

#[test]
fn test_test_codepath_codepath_does_not_fail() {
    fn do_something() -> Result<(), Error> {
        Ok(())
    }

    fn do_failpoint() -> Result<(), Error> {
        _ = failpoint!(do_something(), [Error::msg("Error 1")]);
	Ok(())
    }

    let res = test_codepath! {
            do_failpoint()
    };

    assert!(!res.success());

    assert_eq!(0, res.trigger_count);
    assert_eq!(1, res.expected_trigger_count);
    assert!(res.unexpected_result.is_some());
}
