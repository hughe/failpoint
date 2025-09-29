/// Check everything works if the library is disabled.
use anyhow;

use failpoint::{failpoint, get_count, is_enabled, start_counter};

fn do_something_important() -> Result<(), anyhow::Error> {
    Ok(())
}

fn do_something_else() -> Result<(), anyhow::Error> {
    let res = failpoint!(do_something_important(), [anyhow::Error::msg("Error 1")])?;

    _ = res;

    Ok(())
}

fn main() {
    if is_enabled() {
	println!("failpoint! is enabled");
    } else {
	println!("failpoint! is disabled");
    }
    start_counter();

    let res = do_something_else();

    assert!(res.is_ok());

    if is_enabled() {
	assert_eq!(1, get_count());
    } else {
	assert_eq!(0, get_count());
    }

}
