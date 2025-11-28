/// Injects a failpoint into code for testing error conditions.
///
/// The `failpoint!` macro takes an identifier (usually the result of
/// a function call) that returns a `Result<T, E>` and provides a
/// mechanism to inject errors during testing. The macro also takes an
/// error value that can be returned instead of the original result.
///
/// # Arguments
///
/// * `$res` - An identifier that has the type `Result<T, E>`
/// * `$err` - An error expression of type `E`
/// * `$desc` - An optional description string for logging
///
/// # Modes
///
/// The failpoint macro operates in two modes:
///
/// - **Count mode**: Counts how many failpoints exist in a code path
///   without triggering errors
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
/// let result = do_something();
/// let result = failpoint!(result, Error::msg("Test error"));
/// assert!(result.is_ok());
/// assert_eq!(get_count(), 1);
///
/// // Trigger mode: trigger the first error
/// start_trigger(1);
/// let result = do_something();
/// let result = failpoint!(result, Error::msg("Test error"));
/// assert!(result.is_err());
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
/// let result = do_something();
/// let result = failpoint!(result, Error::msg("Connection failed"), "Database connection");
/// assert!(result.is_err());
/// ```
#[cfg(feature = "failpoint_enabled")]
#[macro_export]
macro_rules! failpoint {
    ($res: ident, $err: expr, $desc: expr) => {{
	failpoint!(@internal $res, $err, Some($desc))
    }};

    ($res: ident, $err: expr) => {{
	failpoint!(@internal $res, $err, None)
    }};

    (@internal $res: ident, $err: expr, $desc_opt: expr) => {{
        {
            let res_ = $res;

            use failpoint::{Mode, lock_state};
            const CRATE_NAME: Option<&'static str> = core::option_env!("CARGO_CRATE_NAME");
            let mut g = lock_state();

            if g.mode == Mode::Count {
                g.counter += 1;
                res_
            } else {
                g.trigger -= 1;
                if g.trigger == 0 {
                    if res_.is_ok() {
                        g.report_trigger(CRATE_NAME, file!(), line!(), $desc_opt, 2);
                        Err($err)
                    } else {
                        g.report_unexpected_failure(CRATE_NAME, file!(), line!(), $desc_opt);
                        res_
                    }
                } else {
                    res_
                }
            }
        }
    }};
}

#[cfg(not(feature = "failpoint_enabled"))]
#[macro_export]
macro_rules! failpoint {
    ($res: ident, $err: expr, $desc: expr) => {{
        let _ = (|| $err);
        $res
    }};

    ($res: ident, $err: expr) => {{
        let _ = (|| $err);
        $res
    }};
}
