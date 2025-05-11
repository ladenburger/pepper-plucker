use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewColor {
    /// String of the hexadecimal representation of the color without `#` (len: 6)
    pub hexadecimal: String,

    /// Defining name of the color (e.g. Red, Orange, ...)
    pub name: String,
}
