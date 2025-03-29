use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Color {
    pub id: i32,
    pub name: String,
    pub hexadecimal: String,
}
