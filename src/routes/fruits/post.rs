use crate::domain::NewFruit;
use crate::routes::common::rollback_and_respond;
use actix_web::{HttpResponse, web};
use bigdecimal::FromPrimitive;
use sqlx::{PgPool, types::BigDecimal};

pub async fn insert_fruit(pool: web::Data<PgPool>, new_fruit: web::Json<NewFruit>) -> HttpResponse {
    if new_fruit.new_color.is_some() && new_fruit.existing_color.is_some() {
        return HttpResponse::BadRequest()
            .body("Cannot provide both new_color and existing_color_id.");
    }

    let mut transaction = pool.begin().await.unwrap();
    let color_id: i32;
    if new_fruit.existing_color.is_some() {
        color_id = new_fruit.existing_color.unwrap();
    } else if new_fruit.new_color.is_none() {
        return rollback_and_respond(transaction).await;
    } else {
        let new_color = new_fruit.new_color.as_ref().unwrap();
        let color_result = sqlx::query!(
            "insert into color (hexadecimal, name) values ($1, $2) returning color_id",
            new_color.hexadecimal,
            new_color.name
        )
        .fetch_one(transaction.as_mut())
        .await;

        match color_result {
            Ok(record) => {
                color_id = record.color_id;
                record
            }
            Err(_e) => return rollback_and_respond(transaction).await,
        };
    }

    let weight: BigDecimal = match BigDecimal::from_f64(new_fruit.average_weight_in_grams) {
        Some(v) => v,
        None => {
            return rollback_and_respond(transaction).await;
        }
    };

    let fruit_result = sqlx::query!(
        "insert into fruit (fruit_name, color, scoville_range_start, scoville_range_end, avg_weight_in_grams, description) 
         values ($1, $2, $3, $4, $5, $6) returning fruit_id",
        new_fruit.fruit_name,
        color_id,
        new_fruit.scoville_start,
        new_fruit.scoville_end,
        weight,
        new_fruit.description
    )
    .fetch_one(transaction.as_mut())
    .await;

    match fruit_result {
        Ok(res) => res.fruit_id,
        Err(_e) => return rollback_and_respond(transaction).await,
    };

    println!("yep im here");

    transaction.commit().await.unwrap();

    HttpResponse::Ok().body("Success! Fruit created.")
}
