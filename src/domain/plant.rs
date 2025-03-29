use super::Fruit;
use chrono::NaiveDate;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Plant {
    pub id: String,
    pub fruit: Fruit,
    pub total_produced_in_grams: f64,
    pub harvests: HashMap<NaiveDate, f64>,
}
