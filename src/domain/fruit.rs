use super::Color;
use bigdecimal::BigDecimal;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Fruit {
    pub id: i32,
    pub name: String,
    pub total_produced_in_grams: BigDecimal,
    pub avg_weight_in_grams: BigDecimal,
    pub color: Color,
}
