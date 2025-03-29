use chrono::NaiveDate;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewHarvest {
    pub date: NaiveDate,

    /// [key]   primary key of plant (example: `"FAT26020012"`)
    /// [value] harvested fruits in grams
    pub plants: HashMap<String, f64>,
    pub notes: Option<String>,
}
