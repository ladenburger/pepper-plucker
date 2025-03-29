use crate::domain::NewPlant;
use actix_web::HttpResponse;
use actix_web::web;
use actix_web::web::Json;
use sqlx::PgPool;

pub async fn insert_plant(pool: web::Data<PgPool>, new_plant: Json<NewPlant>) -> HttpResponse {
    let query_result = sqlx::query!(
        "insert into plant (planted, fruit) values ($1, $2) returning plant_id",
        new_plant.planted,
        new_plant.fruit
    )
    .fetch_one(pool.as_ref())
    .await;

    let plant_id: String = match query_result {
        Ok(r) => r.plant_id,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body("Error creating plant. ".to_owned() + &e.to_string());
        }
    };

    return HttpResponse::Ok().body(plant_id);
}
