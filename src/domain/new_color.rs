use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewColor {
    /// String of the hexadecimal representation of the color without `#` (len: 6)
    pub hexadecimal: String,

    /// [key]   locale_id (example: `"en_US"`, or `"de_DE"`)
    /// [value] color_name (example: `"Red"`, or `"Rot"`)
    pub lang: HashMap<String, String>,
}
