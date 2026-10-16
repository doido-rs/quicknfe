//! Próxima execução diária do sync de catálogo (America/Sao_Paulo, UTC-3).

use chrono::{DateTime, Duration, FixedOffset, NaiveTime, TimeZone, Utc};

use crate::settings;

/// Próximo instante UTC para rodar o job diário.
pub fn next_daily_catalog_run(from: DateTime<Utc>) -> DateTime<Utc> {
    let hour = settings::get().nfse_catalog.daily_hour.min(23);
    let sp = FixedOffset::west_opt(3 * 3600).expect("offset SP");
    let now_sp = from.with_timezone(&sp);
    let time = NaiveTime::from_hms_opt(hour, 0, 0).unwrap_or(NaiveTime::MIN);
    let mut date = now_sp.date_naive();
    let mut target = sp
        .from_local_datetime(&date.and_time(time))
        .single()
        .unwrap_or_else(|| now_sp + Duration::hours(24));
    if target <= now_sp {
        date += Duration::days(1);
        target = sp
            .from_local_datetime(&date.and_time(time))
            .single()
            .unwrap_or_else(|| now_sp + Duration::hours(24));
    }
    target.with_timezone(&Utc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn next_run_is_in_future() {
        let from = Utc.with_ymd_and_hms(2026, 10, 16, 16, 0, 0).unwrap();
        let next = next_daily_catalog_run(from);
        assert!(next > from);
    }
}
