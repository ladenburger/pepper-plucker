use crate::domain::NewHarvest;
use actix_web::HttpResponse;
use actix_web::web;
use bigdecimal::FromPrimitive;
use sqlx::{PgPool, types::BigDecimal};

pub async fn insert_harvest(
    pool: web::Data<PgPool>,
    new_harvest: web::Json<NewHarvest>,
) -> HttpResponse {
    let mut transaction = pool.begin().await.unwrap();
    let harvest_id: i32 = match sqlx::query!(
        "insert into harvest (harvest_date, notes) values ($1, $2) returning harvest_id;",
        new_harvest.date,
        new_harvest.notes,
    )
    .fetch_one(transaction.as_mut())
    .await
    {
        Ok(record) => record.harvest_id,
        Err(e) => {
            transaction.rollback().await.unwrap();

            return HttpResponse::InternalServerError()
                .body("Error creating harvest. ".to_owned() + &e.to_string());
        }
    };

    for (plant_id, weight) in &new_harvest.plants {
        let weight: BigDecimal = match BigDecimal::from_f64(*weight) {
            Some(v) => v,
            None => {
                transaction.rollback().await.unwrap();

                return HttpResponse::BadRequest()
                    .body("Failed to convert f64 into BigDecimal (new_harvest.weight_in_grams)");
            }
        };
        match sqlx::query!(
            "insert into harvest_plant (harvest, plant, weight_in_grams) values ($1, $2, $3)",
            harvest_id,
            plant_id,
            weight
        )
        .execute(transaction.as_mut())
        .await
        {
            Ok(..) => (),
            Err(e) => {
                transaction.rollback().await.unwrap();

                return HttpResponse::InternalServerError()
                    .body("Error inserting plant for harvest. ".to_owned() + &e.to_string());
            }
        };
    }
    transaction.commit().await.unwrap();

    HttpResponse::Ok().body("Harvest created!")
}
