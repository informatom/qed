//! Calendar dates without a date library: today's date, dates as RSS
//! wants them, and dates for readers in the site's language. All in UTC.
//!
//! The conversions between days since 1970 and calendar dates are Howard
//! Hinnant's, see <https://howardhinnant.github.io/date_algorithms.html>.

use std::time::{SystemTime, UNIX_EPOCH};

/// A calendar date. Fields in this order, so dates compare by time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    pub year: i64,
    pub month: u32,
    pub day: u32,
}

/// Seconds since 1970 (UTC).
pub fn now_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |time| i64::try_from(time.as_secs()).unwrap_or(0))
}

/// Today, in UTC.
pub fn today() -> Date {
    from_days(now_seconds().div_euclid(86_400))
}

/// The date `days` days after 1970-01-01.
pub fn from_days(days: i64) -> Date {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    Date {
        year: year_of_era + era * 400 + i64::from(month <= 2),
        // Both are small and positive by construction.
        month: u32::try_from(month).unwrap_or(1),
        day: u32::try_from(day).unwrap_or(1),
    }
}

/// Days since 1970-01-01; the reverse of [`from_days`].
pub fn days(date: Date) -> i64 {
    let year = date.year - i64::from(date.month <= 2);
    let era = year.div_euclid(400);
    let year_of_era = year.rem_euclid(400);
    let month = i64::from(date.month);
    let day_of_year =
        (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + i64::from(date.day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// English month names, short: `Jan` … `Dec`.
const SHORT_MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// The short English name of a date's month: `Oct`.
pub fn short_month(date: Date) -> &'static str {
    SHORT_MONTHS[usize::try_from(date.month.clamp(1, 12) - 1).unwrap_or(0)]
}

/// A date the way RSS writes it (RFC 822): `Wed, 21 Oct 2026 00:00:00 +0000`.
pub fn rfc822(date: Date) -> String {
    const WEEKDAYS: [&str; 7] = ["Thu", "Fri", "Sat", "Sun", "Mon", "Tue", "Wed"];
    // 1970-01-01 was a Thursday.
    let weekday = WEEKDAYS[usize::try_from(days(date).rem_euclid(7)).unwrap_or(0)];
    let month = short_month(date);
    format!(
        "{weekday}, {:02} {month} {:04} 00:00:00 +0000",
        date.day, date.year
    )
}

/// A date for readers, in the site's language (`language` in `site.toml`):
/// `24. Februar 2017` for `de`, `24. Jänner 2017` for `de-AT`,
/// `February 24, 2017` for `en`, `24 February 2017` for `en-GB`. Languages
/// q.e.d. doesn't know get `2017-02-24`.
pub fn long_date(date: Date, language: &str) -> String {
    let language = language.trim().to_lowercase().replace('_', "-");
    let (main, region) = language.split_once('-').unwrap_or((&language, ""));
    let Date { year, day, .. } = date;
    let month = |names: [&'static str; 12]| {
        names[usize::try_from(date.month.clamp(1, 12) - 1).unwrap_or(0)]
    };
    match main {
        "de" => {
            let mut names = [
                "Januar",
                "Februar",
                "März",
                "April",
                "Mai",
                "Juni",
                "Juli",
                "August",
                "September",
                "Oktober",
                "November",
                "Dezember",
            ];
            if region == "at" {
                names[0] = "Jänner";
            }
            format!("{day}. {} {year}", month(names))
        }
        "en" => {
            let name = month([
                "January",
                "February",
                "March",
                "April",
                "May",
                "June",
                "July",
                "August",
                "September",
                "October",
                "November",
                "December",
            ]);
            // Day first everywhere English is written but in the US.
            if matches!(region, "" | "us") {
                format!("{name} {day}, {year}")
            } else {
                format!("{day} {name} {year}")
            }
        }
        "fr" => {
            let name = month([
                "janvier",
                "février",
                "mars",
                "avril",
                "mai",
                "juin",
                "juillet",
                "août",
                "septembre",
                "octobre",
                "novembre",
                "décembre",
            ]);
            let day = if day == 1 {
                "1er".to_string()
            } else {
                day.to_string()
            };
            format!("{day} {name} {year}")
        }
        "es" => format!(
            "{day} de {} de {year}",
            month([
                "enero",
                "febrero",
                "marzo",
                "abril",
                "mayo",
                "junio",
                "julio",
                "agosto",
                "septiembre",
                "octubre",
                "noviembre",
                "diciembre",
            ])
        ),
        "it" => format!(
            "{day} {} {year}",
            month([
                "gennaio",
                "febbraio",
                "marzo",
                "aprile",
                "maggio",
                "giugno",
                "luglio",
                "agosto",
                "settembre",
                "ottobre",
                "novembre",
                "dicembre",
            ])
        ),
        "nl" => format!(
            "{day} {} {year}",
            month([
                "januari",
                "februari",
                "maart",
                "april",
                "mei",
                "juni",
                "juli",
                "augustus",
                "september",
                "oktober",
                "november",
                "december",
            ])
        ),
        "pt" => format!(
            "{day} de {} de {year}",
            month([
                "janeiro",
                "fevereiro",
                "março",
                "abril",
                "maio",
                "junho",
                "julho",
                "agosto",
                "setembro",
                "outubro",
                "novembro",
                "dezembro",
            ])
        ),
        _ => format!("{year:04}-{:02}-{day:02}", date.month),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_dates_in_the_site_language() {
        let date = |year, month, day| Date { year, month, day };
        let feb = date(2017, 2, 24);
        assert_eq!(long_date(feb, "de"), "24. Februar 2017");
        assert_eq!(long_date(date(2019, 1, 18), "de"), "18. Januar 2019");
        assert_eq!(long_date(date(2019, 1, 18), "de-AT"), "18. Jänner 2019");
        assert_eq!(long_date(date(2019, 1, 18), "de_at"), "18. Jänner 2019");
        assert_eq!(long_date(feb, "en"), "February 24, 2017");
        assert_eq!(long_date(feb, "en-GB"), "24 February 2017");
        assert_eq!(long_date(date(2026, 3, 1), "fr"), "1er mars 2026");
        assert_eq!(long_date(feb, "es"), "24 de febrero de 2017");
        assert_eq!(long_date(date(2014, 3, 3), "xx"), "2014-03-03");
    }

    fn date(year: i64, month: u32, day: u32) -> Date {
        Date { year, month, day }
    }

    #[test]
    fn converts_both_ways() {
        assert_eq!(from_days(0), date(1970, 1, 1));
        assert_eq!(from_days(11_016), date(2000, 2, 29));
        for days_since in [-1000, 0, 11_016, 20_747, 100_000] {
            assert_eq!(days(from_days(days_since)), days_since);
        }
    }

    #[test]
    fn writes_rss_dates() {
        assert_eq!(
            rfc822(date(2026, 10, 21)),
            "Wed, 21 Oct 2026 00:00:00 +0000"
        );
        assert_eq!(rfc822(date(1970, 1, 1)), "Thu, 01 Jan 1970 00:00:00 +0000");
        assert_eq!(rfc822(date(2000, 2, 29)), "Tue, 29 Feb 2000 00:00:00 +0000");
    }
}
