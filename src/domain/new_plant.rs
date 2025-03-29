use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewPlant {
    /// Date when the plant was potted.
    pub planted: NaiveDate,

    /// Which type of fruit does the plant produce
    pub fruit: i32,
}
