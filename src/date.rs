use anyhow::{Context, Result};
use std::time::{SystemTime, UNIX_EPOCH};

/// Format the UTC calendar date of `time` as `YYYY-MM-DD`.
///
/// Time is injected rather than read from the wall clock here, so callers stay
/// deterministic and testable — the same pattern `checkpoint` uses for its
/// timestamp.
pub(crate) fn today_utc(time: SystemTime) -> Result<String> {
    let duration = time
        .duration_since(UNIX_EPOCH)
        .context("system time is before Unix epoch")?;
    let days = (duration.as_secs() / 86_400) as i64;
    let (year, month, day) = civil_from_days(days);
    Ok(format!("{year:04}-{month:02}-{day:02}"))
}

/// Convert days since the Unix epoch into a `(year, month, day)` civil date
/// (Howard Hinnant's `civil_from_days`).
pub(crate) fn civil_from_days(days_since_epoch: i64) -> (i32, u32, u32) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    let year = year + if month <= 2 { 1 } else { 0 };
    (year as i32, month as u32, day as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn today_utc_formats_the_unix_epoch() {
        assert_eq!(today_utc(UNIX_EPOCH).expect("epoch date"), "1970-01-01");
    }

    #[test]
    fn today_utc_advances_at_the_day_boundary() {
        let next_day = UNIX_EPOCH + Duration::from_secs(86_400);
        assert_eq!(today_utc(next_day).expect("next day"), "1970-01-02");
    }
}
