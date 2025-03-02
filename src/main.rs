use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use bigdecimal::FromPrimitive;
use chrono::Local;
use chrono::NaiveDate;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions, types::BigDecimal};
use std::collections::HashMap;
use std::env;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Color {
    /// primarykey/unique-id of a color
    color_id: i32,

    /// String of the hexadecimal representation of the color.
    hexadecimal: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct NewColor {
    /// String of the hexadecimal representation of the color without `#` (len: 6)
    hexadecimal: String,

    /// [key]   locale_id (example: `"en_US"`, or `"de_DE"`)
    /// [value] color_name (example: `"Red"`, or `"Rot"`)
    lang: HashMap<String, String>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct NewFruit {
    /// Primary key of color entity (cannot be used in conjunction with new_color).
    /// **Example:** `5`, which would reference to entity `5 | 0F41A1 |`
    existing_color: Option<i32>,

    /// Properties for the new color entity (cannot be used in conjunction with existing_color).
    new_color: Option<NewColor>,

    /// Primary key of fruit_type entity (cannot be used in conjunction with new_fruit_type).
    /// **Example:** `15`, which would reference to entity `15 | Habanero |`
    existing_fruit_type: Option<i32>,

    /// Name for the new fruit_type entity (cannot be used in conjunction with
    /// existing_fruit_type).
    /// **Example:** `"Habanero"` or `"Chilli"`
    new_fruit_type: Option<String>,

    /// Name of the fruit, **NOT species/type**
    /// **Example:** `"Fatalii"`
    fruit_name: String,

    /// Scovillerange start_value, or plain scoville value if no range_end is specified.
    /// **Unit:** `SHU` (Scoville heat units)
    scoville_start: i32,

    /// Optionally define a scoville_range_end to interpret entries as a range of values.
    /// **Unit:** `SHU` (Scoville heat units)
    scoville_end: Option<i32>,

    /// Average weight of a single fruit in grams.
    average_weight_in_grams: f64,

    fruit_descriptions: Option<HashMap<String, String>>,
    fruit_type_descriptions: Option<HashMap<String, String>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct NewPlant {
    planted: NaiveDate,
    fruit: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct NewHarvest {
    date: NaiveDate,
    /// [key]   primary key of plant (example: `"FAT26020012"`)
    /// [value] harvested fruits in grams
    plants: HashMap<String, f64>,
    notes: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DisposePlant {
    date_disposed: Option<NaiveDate>,
}

async fn insert_fruit(pool: web::Data<PgPool>, new_fruit: web::Json<NewFruit>) -> impl Responder {
    if new_fruit.new_color.is_some() && new_fruit.existing_color.is_some() {
        return HttpResponse::BadRequest()
            .body("Cannot provide both new_color and existing_color_id.");
    }

    if new_fruit.new_fruit_type.is_some() && new_fruit.existing_fruit_type.is_some() {
        return HttpResponse::BadRequest()
            .body("Cannot provide both new_fruit_type and existing_fruit_type.");
    }

    if new_fruit.existing_fruit_type.is_some() && new_fruit.fruit_type_descriptions.is_some() {
        return HttpResponse::BadRequest().body("Cannot add description to existing fruit_type");
    }

    let mut transaction = pool.begin().await.unwrap();
    let color_id: i32;

    if let Some(new_color) = &new_fruit.new_color {
        let color_result = sqlx::query!(
            "INSERT INTO color (hexadecimal) VALUES ($1) RETURNING color_id",
            new_color.hexadecimal
        )
        .fetch_one(transaction.as_mut())
        .await;

        match color_result {
            Ok(record) => {
                color_id = record.color_id;
                for (lang_code, color_name) in &new_color.lang {
                    let insert_result = sqlx::query!(
                        "INSERT INTO locale_color (color, locale_id, value) VALUES ($1, $2, $3)",
                        color_id,
                        lang_code,
                        color_name
                    )
                    .execute(transaction.as_mut())
                    .await;

                    match insert_result {
                        Ok(..) => (),
                        Err(e) => {
                            transaction.rollback().await.unwrap();

                            return HttpResponse::InternalServerError()
                                .body("Error creating color_locale. ".to_owned() + &e.to_string());
                        }
                    }
                }
            }
            Err(e) => {
                transaction.rollback().await.unwrap();

                return HttpResponse::InternalServerError().body(String::from(
                    "Failed to create color: Database insert operation failed. ".to_owned()
                        + &e.to_string(),
                ));
            }
        }
    } else if let Some(existing_color_id) = new_fruit.existing_color {
        color_id = existing_color_id;
    } else {
        transaction.rollback().await.unwrap();

        return HttpResponse::BadRequest()
            .body("Either new_color or existing_color_id must be provided.");
    }

    let fruit_type_id: i32;

    if let Some(new_fruit_type) = &new_fruit.new_fruit_type {
        let fruit_type_result = sqlx::query!(
            "INSERT INTO fruit_type (type_name) VALUES ($1) RETURNING type_id",
            new_fruit_type
        )
        .fetch_one(transaction.as_mut())
        .await;

        match fruit_type_result {
            Ok(record) => {
                fruit_type_id = record.type_id;
                match &new_fruit.fruit_type_descriptions {
                    Some(desc) => {
                        for (lang_code, value) in desc {
                            let insert_result = sqlx::query!(
                                "INSERT INTO locale_fruit_type_desc (locale_id, fruit_type, value) VALUES ($1, $2, $3)",
                                lang_code,
                                fruit_type_id,
                                value
                            )
                            .execute(transaction.as_mut())
                            .await;

                            match insert_result {
                                Ok(..) => (),
                                Err(e) => {
                                    transaction.rollback().await.unwrap();

                                    return HttpResponse::InternalServerError().body(
                                        "Error creating description. ".to_owned() + &e.to_string(),
                                    );
                                }
                            }
                        }
                    }
                    None => (),
                };
            }
            Err(e) => {
                transaction.rollback().await.unwrap();

                return HttpResponse::InternalServerError().body(String::from(
                    "Failed to create fruit_type: Database insert operation failed. ".to_owned()
                        + &e.to_string(),
                ));
            }
        }
    } else if let Some(existing_fruit_type) = new_fruit.existing_fruit_type {
        fruit_type_id = existing_fruit_type;
    } else {
        return HttpResponse::BadRequest()
            .body("Either new_fruit_type or existing_fruit_type must be provided.");
    }

    let weight: BigDecimal = match BigDecimal::from_f64(new_fruit.average_weight_in_grams) {
        Some(v) => v,
        None => {
            transaction.rollback().await.unwrap();

            return HttpResponse::BadRequest()
                .body("Failed to convert f64 into BigDecimal (new_fruit.average_weight_in_grams");
        }
    };

    let fruit_result = sqlx::query!(
        "INSERT INTO fruit (fruit_type, fruit_name, color, scoville_range_start, scoville_range_end, avg_weight_in_grams) 
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING fruit_id",
        fruit_type_id,
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
        Err(e) => {
            transaction.rollback().await.unwrap();

            return HttpResponse::InternalServerError().body(String::from(
                "Failed to create fruit: Database insert operation failed. ".to_owned()
                    + &e.to_string(),
            ));
        }
    };

    match &new_fruit.fruit_descriptions {
        Some(desc) => {
            for (lang_code, value) in desc {
                let insert_result = sqlx::query!(
                    "INSERT INTO locale_fruit_desc (locale_id, fruit, value) VALUES ($1, $2, $3)",
                    lang_code,
                    fruit_id,
                    value
                )
                .execute(transaction.as_mut())
                .await;

                match insert_result {
                    Ok(..) => (),
                    Err(e) => {
                        transaction.rollback().await.unwrap();

                        return HttpResponse::InternalServerError()
                            .body("Error creating description. ".to_owned() + &e.to_string());
                    }
                }
            }
        }
        None => (),
    };

    transaction.commit().await.unwrap();

    HttpResponse::Ok().body("")
}

async fn insert_plant(pool: web::Data<PgPool>, new_plant: web::Json<NewPlant>) -> impl Responder {
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

async fn dispose_plant(
    pool: web::Data<PgPool>,
    path: web::Path<(String,)>,
    body: web::Json<DisposePlant>,
) -> impl Responder {
    println!("hi");
    let plant_id: String = path.into_inner().0;
    let date = match body.date_disposed {
        Some(date) => date,
        None => Local::now().naive_utc().into(),
    };

    match sqlx::query!(
        "update plant set disposed = $1 where plant_id = $2 and disposed is null",
        date,
        plant_id
    )
    .execute(pool.as_ref())
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::InternalServerError()
            .body("Error updating disposed state. ".to_owned() + &e.to_string()),
    }
}

// TODO: implement
async fn insert_harvest() -> impl Responder {
    HttpResponse::Ok().body("Hi")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool.");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("fruit", web::post().to(insert_fruit))
            .route("plant", web::post().to(insert_plant))
            .route("plant/dispose/{plant_id}", web::patch().to(dispose_plant))
            .route("harvest", web::post().to(insert_harvest))
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}
