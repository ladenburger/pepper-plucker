use super::new_color::NewColor;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewFruit {
    /// Primary key of color entity (cannot be used in conjunction with new_color).
    /// **Example:** `5`, which would reference to entity `5 | 0F41A1 |`
    pub existing_color: Option<i32>,

    /// Properties for the new color entity (cannot be used in conjunction with existing_color).
    pub new_color: Option<NewColor>,

    /// Name of the fruit, **NOT species/type**
    /// **Example:** `"Fatalii"`
    pub fruit_name: String,

    /// Scovillerange start_value, or plain scoville value if no range_end is specified.
    /// **Unit:** `SHU` (Scoville heat units)
    pub scoville_start: i32,

    /// Optionally define a scoville_range_end to interpret entries as a range of values.
    /// **Unit:** `SHU` (Scoville heat units)
    pub scoville_end: Option<i32>,

    /// Average weight of a single fruit in grams.
    pub average_weight_in_grams: f64,

    /// Optionally add localized descriptions to the new fruit
    pub fruit_descriptions: Option<HashMap<String, String>>,
}
