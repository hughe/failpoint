use std::fmt::Debug;

use crate::failpoint_state::{get_counted_locs, get_triggered_locs};
use crate::{log_if_verbose, Verbosity};

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

impl<T, E> CodePathResult<T, E>
where
    T: Debug,
    E: Debug,
{
    pub fn report(&self, name: &str) {
        use Verbosity;

        log_if_verbose(
            Verbosity::Moderate,
            format!("************************************************************************"),
        );
        log_if_verbose(Verbosity::Moderate, format!("* Codepath:   {name}"));

        log_if_verbose(Verbosity::Moderate, "*".to_string());

        log_if_verbose(
            Verbosity::Moderate,
            format!("* Counted:    {}", self.expected_trigger_count),
        );
        log_if_verbose(
            Verbosity::Moderate,
            format!("* Triggered:  {}", self.trigger_count),
        );
        if let Some(unex) = &self.unexpected_result {
            log_if_verbose(Verbosity::Moderate, format!("* Unexpected: {:?}", unex));
        }

        log_if_verbose(Verbosity::Extreme, "*".to_string());

        let counted_locs = get_counted_locs();
        log_if_verbose(Verbosity::Extreme, format!("* Counted Failpoints: "));

        for (i, loc) in counted_locs.iter().enumerate() {
            log_if_verbose(
                Verbosity::Extreme,
                format!("*   {:3}| {}", i + 1, loc.format()),
            );
        }

        log_if_verbose(Verbosity::Extreme, "*".to_string());
        let triggered_locs = get_triggered_locs();
        log_if_verbose(Verbosity::Extreme, format!("* Triggered Failpoints:"));

        for (i, loc) in triggered_locs.iter().enumerate() {
            log_if_verbose(
                Verbosity::Extreme,
                format!("*   {:3}| {}", i + 1, loc.format()),
            );
        }
        log_if_verbose(Verbosity::Extreme, "*".to_string());

        let ok = if self.success() { "SUCCESS" } else { "FAILED" };
        log_if_verbose(Verbosity::Moderate, format!("* Result:     {ok}"));

        log_if_verbose(
            Verbosity::Moderate,
            format!("************************************************************************"),
        );
    }
}
