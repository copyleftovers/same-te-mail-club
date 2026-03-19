/// Format a UTC `OffsetDateTime` for Ukrainian display.
///
/// Produces e.g. "25 березня 2026, 21:09".
pub fn format_date_uk(dt: time::OffsetDateTime) -> String {
    let month_name = match dt.month() {
        time::Month::January => "січня",
        time::Month::February => "лютого",
        time::Month::March => "березня",
        time::Month::April => "квітня",
        time::Month::May => "травня",
        time::Month::June => "червня",
        time::Month::July => "липня",
        time::Month::August => "серпня",
        time::Month::September => "вересня",
        time::Month::October => "жовтня",
        time::Month::November => "листопада",
        time::Month::December => "грудня",
    };
    format!(
        "{} {} {}, {:02}:{:02}",
        dt.day(),
        month_name,
        dt.year(),
        dt.hour(),
        dt.minute(),
    )
}
