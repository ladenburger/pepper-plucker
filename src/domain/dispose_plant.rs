use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisposePlant {
    pub date_disposed: Option<NaiveDate>,
}
