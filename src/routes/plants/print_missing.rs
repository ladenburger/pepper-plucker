use actix_web::{HttpResponse, web};
use askama_actix::{Template, TemplateToResponse};
use sqlx::PgPool;

#[derive(Template)]
#[template(path = "print_plants.html")]
struct PrintPlantHtml {
    plants: Vec<String>,
}

pub async fn print_plant_labels(pool: web::Data<PgPool>) -> HttpResponse {
    let plants = sqlx::query!("select plant_id from plant where is_label_printed = false;")
        .fetch_all(pool.as_ref())
        .await
        .map(|rows| rows.into_iter().map(|row| row.plant_id).collect());

    if plants.is_err() {
        return HttpResponse::InternalServerError().body("");
    }
    let plants: Vec<String> = plants.unwrap();

    if plants.is_empty() {
        return HttpResponse::InternalServerError().body("");
    }
    // TODO: set plantstatus to label_printed

    PrintPlantHtml { plants }.to_response()
}
