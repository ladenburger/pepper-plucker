use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HarvestFruit {
    fruit_id: i32,
    name: String,
    weight: f64,
}
