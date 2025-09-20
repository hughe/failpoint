use anyhow::Error;

use failpoint::{failpoint, get_count, start_counter, start_trigger, test_codepath};

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
    {
            do_failpoint()
    }
    };

    assert!(res.success());

    assert_eq!(1, res.trigger_count);
    assert_eq!(1, res.expected_trigger_count);
    assert!(res.unexpected_result.is_none());
}
