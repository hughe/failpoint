/// Check everything works if the library is disabled.
///
/// Run like this to test with failpoints enabled:
///
/// ```shell
/// cargo run --example conditional_comp
/// ```
///
/// And like this to test with failpoints disabled.
///
/// ```shell
/// cargo run --example conditional_comp --no-default-features
/// ```
use failpoint::failpoint;

fn do_something_important() -> Result<(), anyhow::Error> {
    Ok(())
}

fn code_under_test() -> Result<(), anyhow::Error> {
    let res = do_something_important();
    let res = failpoint!(res, anyhow::Error::msg("Error 1"));

    res
}

fn main() {
    if failpoint::is_enabled() {
        println!("failpoint! is enabled");
    } else {
        println!("failpoint! is disabled");
    }
    failpoint::start_counter();

    let res = code_under_test();

    assert!(res.is_ok());

    if failpoint::is_enabled() {
        assert_eq!(1, failpoint::get_count());
    } else {
        assert_eq!(0, failpoint::get_count());
    }
}
