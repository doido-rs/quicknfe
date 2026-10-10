use chrono::{Datelike, Local, NaiveDate, TimeZone, Utc};
use doido::model::sea_orm::prelude::Decimal;
use doido::model::sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Serialize;

use crate::models::documento_fiscal::{Column as DocColumn, Entity as DocEntity, Model as Documento};

#[derive(Debug, Clone, Serialize)]
pub struct MonthSummary {
    pub year: i32,
    pub month: u32,
    pub note_count: u64,
    pub revenue_total: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DailyComparisonPoint {
    pub day: u32,
    pub previous_month: f64,
    pub current_month: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardStats {
    pub previous_month: MonthSummary,
    pub current_month: MonthSummary,
    pub notes_cumulative_by_day: Vec<DailyComparisonPoint>,
    pub revenue_cumulative_by_day: Vec<DailyComparisonPoint>,
}

fn decimal_to_f64(value: Decimal) -> f64 {
    value.to_string().parse().unwrap_or(0.0)
}

fn month_start_local(year: i32, month: u32) -> chrono::DateTime<Utc> {
    let naive = NaiveDate::from_ymd_opt(year, month, 1)
        .expect("valid month start")
        .and_hms_opt(0, 0, 0)
        .expect("valid time");
    Local
        .from_local_datetime(&naive)
        .single()
        .expect("local month start maps to single instant")
        .with_timezone(&Utc)
}

fn add_months(year: i32, month: u32, delta: i32) -> (i32, u32) {
    let total = year * 12 + month as i32 - 1 + delta;
    ((total / 12), (total % 12 + 1) as u32)
}

fn days_in_month(year: i32, month: u32) -> u32 {
    NaiveDate::from_ymd_opt(
        if month == 12 { year + 1 } else { year },
        if month == 12 { 1 } else { month + 1 },
        1,
    )
    .unwrap()
    .pred_opt()
    .unwrap()
    .day()
}

fn doc_local_date(doc: &Documento) -> NaiveDate {
    doc.data_emissao.with_timezone(&Local).date_naive()
}

fn summarize_month(docs: &[&Documento], year: i32, month: u32) -> MonthSummary {
    let mut note_count = 0u64;
    let mut revenue_total = 0.0f64;
    for doc in docs {
        let d = doc_local_date(doc);
        if d.year() == year && d.month() == month {
            note_count += 1;
            revenue_total += decimal_to_f64(doc.valor_liquido);
        }
    }
    MonthSummary {
        year,
        month,
        note_count,
        revenue_total,
    }
}

fn cumulative_series(
    docs: &[&Documento],
    prev_year: i32,
    prev_month: u32,
    curr_year: i32,
    curr_month: u32,
    today: NaiveDate,
    value_fn: fn(&Documento) -> f64,
) -> Vec<DailyComparisonPoint> {
    let max_day = days_in_month(prev_year, prev_month)
        .max(days_in_month(curr_year, curr_month))
        .max(if curr_year == today.year() && curr_month == today.month() {
            today.day()
        } else {
            days_in_month(curr_year, curr_month)
        });

    let mut daily_prev = vec![0.0f64; max_day as usize + 1];
    let mut daily_curr = vec![0.0f64; max_day as usize + 1];

    for doc in docs {
        let d = doc_local_date(doc);
        let value = value_fn(doc);
        if d.year() == prev_year && d.month() == prev_month {
            daily_prev[d.day() as usize] += value;
        } else if d.year() == curr_year && d.month() == curr_month {
            daily_curr[d.day() as usize] += value;
        }
    }

    let mut cum_prev = 0.0;
    let mut cum_curr = 0.0;
    let mut points = Vec::with_capacity(max_day as usize);

    for day in 1..=max_day {
        cum_prev += daily_prev[day as usize];
        let in_current_month =
            curr_year == today.year() && curr_month == today.month() && day <= today.day();
        let in_past_current =
            curr_year < today.year()
                || (curr_year == today.year() && curr_month < today.month());
        if in_current_month || in_past_current {
            cum_curr += daily_curr[day as usize];
        }
        points.push(DailyComparisonPoint {
            day,
            previous_month: cum_prev,
            current_month: cum_curr,
        });
    }

    points
}

pub async fn load_dashboard_stats(
    db: &doido::model::sea_orm::DatabaseConnection,
    empresa_id: i64,
) -> Result<DashboardStats, doido::model::sea_orm::DbErr> {
    let today = Local::now().date_naive();
    let (curr_year, curr_month) = (today.year(), today.month());
    let (prev_year, prev_month) = add_months(curr_year, curr_month, -1);

    let range_start = month_start_local(prev_year, prev_month);
    let (next_year, next_month) = add_months(curr_year, curr_month, 1);
    let range_end = month_start_local(next_year, next_month);

    let rows = DocEntity::find()
        .filter(DocColumn::EmpresaId.eq(empresa_id))
        .filter(DocColumn::DataEmissao.gte(range_start))
        .filter(DocColumn::DataEmissao.lt(range_end))
        .all(db)
        .await?;

    let doc_refs: Vec<&Documento> = rows.iter().collect();

    let previous_month = summarize_month(&doc_refs, prev_year, prev_month);
    let current_month = summarize_month(&doc_refs, curr_year, curr_month);

    let notes_cumulative_by_day = cumulative_series(
        &doc_refs,
        prev_year,
        prev_month,
        curr_year,
        curr_month,
        today,
        |_| 1.0,
    );

    let revenue_cumulative_by_day = cumulative_series(
        &doc_refs,
        prev_year,
        prev_month,
        curr_year,
        curr_month,
        today,
        |doc| decimal_to_f64(doc.valor_liquido),
    );

    Ok(DashboardStats {
        previous_month,
        current_month,
        notes_cumulative_by_day,
        revenue_cumulative_by_day,
    })
}
