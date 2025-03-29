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
            "insert into color (hexadecimal) values ($1) returning color_id",
            new_color.hexadecimal
        )
        .fetch_one(transaction.as_mut())
        .await;

        match color_result {
            Ok(record) => {
                color_id = record.color_id;
                record
            }
            Err(e) => return rollback_and_respond(transaction).await,
        };

        for (lang_code, color_name) in &new_color.lang {
            let locale_label_content_insert_result = sqlx::query!(
                "insert into localized_text_content
                    (locale_id, label, option_reference_id, value) 
                values ($1, $2, $3, $4)",
                lang_code,
                String::from("FRUIT_COLOR"),
                color_id,
                color_name
            )
            .execute(transaction.as_mut())
            .await;
            if locale_label_content_insert_result.is_err() {
                return rollback_and_respond(transaction).await;
            }
        }
    }

    let weight: BigDecimal = match BigDecimal::from_f64(new_fruit.average_weight_in_grams) {
        Some(v) => v,
        None => {
            return rollback_and_respond(transaction).await;

            // return HttpResponse::BadRequest()
            //     .body("Failed to convert f64 into BigDecimal (new_fruit.average_weight_in_grams)");
        }
    };

    let fruit_result = sqlx::query!(
        "insert into fruit (fruit_name, color, scoville_range_start, scoville_range_end, avg_weight_in_grams) 
         values ($1, $2, $3, $4, $5) returning fruit_id",
        new_fruit.fruit_name,
        color_id,
        new_fruit.scoville_start,
        new_fruit.scoville_end,
        weight
    )
    .fetch_one(transaction.as_mut())
    .await;

    let fruit_id: i32 = match fruit_result {
        Ok(res) => res.fruit_id,
        Err(e) => return rollback_and_respond(transaction).await,
    };

    let descriptions = match &new_fruit.fruit_descriptions {
        // No descriptions? Then we're done.
        None => {
            transaction.commit().await.unwrap();
            return HttpResponse::Ok().body("Fruit created!");
        }
        Some(desc) => desc,
    };

    for (lang_code, value) in descriptions {
        let insert_desc_result = sqlx::query!(
            "insert into localized_text_content
                (locale_id, label, option_reference_id, value)
            values ($1, $2, $3, $4);",
            lang_code,
            String::from("FRUIT_DESCRIPTION"),
            fruit_id,
            value
        )
        .execute(transaction.as_mut())
        .await;
        if insert_desc_result.is_err() {
            return rollback_and_respond(transaction).await;
        }
    }
    transaction.commit().await.unwrap();

    HttpResponse::Ok().body("")
}
