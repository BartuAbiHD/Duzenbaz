//! Ekran süresi ve mola günlüğü. Yalnızca saat bazında süre tutulur.
use crate::error::AppResult;
use chrono::{Datelike, Duration, NaiveDate};
use rusqlite::{params, Connection};
use serde::Serialize;
use std::collections::HashMap;

pub type Pending = HashMap<(String, u32), u32>;

/// Bellekte biriken saniyeleri veritabanına yazar ve birikimi sıfırlar.
pub fn flush(conn: &mut Connection, pending: &mut Pending) -> AppResult<()> {
    if pending.is_empty() {
        return Ok(());
    }
    let tx = conn.transaction()?;
    for ((day, hour), secs) in pending.iter() {
        tx.execute(
            "INSERT INTO screen_time (day, hour, seconds) VALUES (?1, ?2, ?3)
             ON CONFLICT(day, hour) DO UPDATE SET seconds = seconds + excluded.seconds",
            params![day, hour, secs],
        )?;
    }
    tx.commit()?;
    pending.clear();
    Ok(())
}

pub fn log_break(conn: &Connection, day: &str, completed: bool) -> AppResult<()> {
    let col = if completed { "taken" } else { "skipped" };
    conn.execute(
        &format!("INSERT INTO break_log (day, {col}) VALUES (?1, 1) ON CONFLICT(day) DO UPDATE SET {col} = {col} + 1"),
        [day],
    )?;
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct DayTotal {
    pub day: String,
    pub seconds: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    /// Bugünün 24 saati için saniye.
    pub hours: Vec<u32>,
    /// İçinde bulunulan haftanın 7 günü (hafta başlangıcından itibaren); eksik ve gelecek günler 0.
    pub days: Vec<DayTotal>,
    /// `days` içinde bugünün sırası.
    pub today_index: usize,
    pub breaks_taken: u32,
    pub breaks_skipped: u32,
}

/// `week_start`: haftanın ilk günü, 0 = Pazar, 1 = Pazartesi (Türkiye).
pub fn build_report(conn: &Connection, today: NaiveDate, week_start: u32) -> AppResult<Report> {
    let day_str = |d: NaiveDate| d.format("%Y-%m-%d").to_string();
    let offset = (today.weekday().num_days_from_sunday() + 7 - week_start % 7) % 7;
    let start = today - Duration::days(offset as i64);

    let mut hours = vec![0u32; 24];
    let mut stmt = conn.prepare("SELECT hour, seconds FROM screen_time WHERE day = ?1")?;
    for row in stmt.query_map([day_str(today)], |r| Ok((r.get::<_, u32>(0)?, r.get::<_, u32>(1)?)))? {
        let (h, s) = row?;
        if let Some(slot) = hours.get_mut(h as usize) {
            *slot = s;
        }
    }

    let from = day_str(start);
    let mut totals: HashMap<String, u32> = HashMap::new();
    let mut stmt = conn.prepare("SELECT day, SUM(seconds) FROM screen_time WHERE day >= ?1 GROUP BY day")?;
    for row in stmt.query_map([from], |r| Ok((r.get::<_, String>(0)?, r.get::<_, u32>(1)?)))? {
        let (d, s) = row?;
        totals.insert(d, s);
    }
    let days = (0..7)
        .map(|i| {
            let day = day_str(start + Duration::days(i));
            DayTotal { seconds: totals.get(&day).copied().unwrap_or(0), day }
        })
        .collect();

    let (breaks_taken, breaks_skipped) = conn
        .query_row("SELECT taken, skipped FROM break_log WHERE day = ?1", [day_str(today)], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap_or((0, 0));
    Ok(Report { hours, days, today_index: offset as usize, breaks_taken, breaks_skipped })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db() -> Connection {
        let mut c = Connection::open_in_memory().unwrap();
        crate::db::migrate(&mut c).unwrap();
        c
    }

    #[test]
    fn flush_accumulates_and_report_zero_fills() {
        let mut c = db();
        let today = NaiveDate::from_ymd_opt(2026, 10, 30).unwrap();
        let mut p = Pending::new();
        p.insert(("2026-10-30".into(), 9), 600);
        p.insert(("2026-10-30".into(), 10), 300);
        p.insert(("2026-10-28".into(), 14), 1800);
        flush(&mut c, &mut p).unwrap();
        assert!(p.is_empty());
        p.insert(("2026-10-30".into(), 9), 60);
        flush(&mut c, &mut p).unwrap();

        // 30 Ekim 2026 Cuma; hafta Pazartesi'den başlar.
        let r = build_report(&c, today, 1).unwrap();
        assert_eq!((r.hours[9], r.hours[10], r.hours[11]), (660, 300, 0));
        assert_eq!(r.days.len(), 7);
        assert_eq!(r.days[0].day, "2026-10-26");
        assert_eq!(r.today_index, 4);
        assert_eq!(r.days[4].seconds, 960);
        assert_eq!(r.days[2].seconds, 1800, "28 Ekim Çarşamba");
        assert_eq!(r.days[3].seconds, 0);
        assert_eq!(r.days[6].day, "2026-11-01", "gelecek günler de listelenir");

        let sunday_first = build_report(&c, today, 0).unwrap();
        assert_eq!(sunday_first.days[0].day, "2026-10-25");
        assert_eq!(sunday_first.today_index, 5);
    }

    #[test]
    fn sunday_belongs_to_the_end_of_a_monday_week() {
        let c = db();
        let r = build_report(&c, NaiveDate::from_ymd_opt(2026, 11, 1).unwrap(), 1).unwrap();
        assert_eq!((r.days[0].day.as_str(), r.today_index), ("2026-10-26", 6));
    }

    #[test]
    fn break_log_counts() {
        let c = db();
        log_break(&c, "2026-10-30", true).unwrap();
        log_break(&c, "2026-10-30", true).unwrap();
        log_break(&c, "2026-10-30", false).unwrap();
        let r = build_report(&c, NaiveDate::from_ymd_opt(2026, 10, 30).unwrap(), 1).unwrap();
        assert_eq!((r.breaks_taken, r.breaks_skipped), (2, 1));
    }
}
