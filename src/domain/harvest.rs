use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Harvest {
    id: i32,
    plants: HashMap<String, f64>,
    fruits: Vec<HarvestFruit>,
}
