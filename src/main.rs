use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use askama_actix::{Template, TemplateToResponse};
use bigdecimal::FromPrimitive;
use chrono::Local;
use chrono::NaiveDate;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions, types::BigDecimal};
use std::collections::HashMap;
use std::env;

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

    /// Optionally add localized descriptions to the new fruit
    fruit_descriptions: Option<HashMap<String, String>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct NewPlant {
    /// Date when the plant was potted.
    planted: NaiveDate,

    /// Which type of fruit does the plant produce
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Color {
    id: i32,
    name: String,
    hexadecimal: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Fruit {
    id: i32,
    name: String,
    total_produced_in_grams: BigDecimal,
    avg_weight_in_grams: BigDecimal,
    color: Color,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Plant {
    id: String,
    fruit: Fruit,
    total_produced_in_grams: f64,
    harvests: HashMap<NaiveDate, f64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HarvestFruit {
    fruit_id: i32,
    name: String,
    weight: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Harvest {
    id: i32,
    plants: HashMap<String, f64>,
    fruits: Vec<HarvestFruit>,
}

async fn rollback_and_respond(transaction: sqlx::Transaction<'_, sqlx::Postgres>) -> HttpResponse {
    transaction.rollback().await.unwrap();

    return HttpResponse::InternalServerError().body(String::from(
        "Failed to create color: Database insert operation failed. ".to_owned(),
    ));
}

async fn insert_fruit(pool: web::Data<PgPool>, new_fruit: web::Json<NewFruit>) -> impl Responder {
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

async fn insert_harvest(
    pool: web::Data<PgPool>,
    new_harvest: web::Json<NewHarvest>,
) -> impl Responder {
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

async fn select_fruits(pool: web::Data<PgPool>) -> impl Responder {
    let fruits = sqlx::query!(
        "select
           f.fruit_id,
           f.fruit_name,
           f.avg_weight_in_grams,
           f.scoville_range_start,
           f.scoville_range_end,
           (select count(plant.plant_id) from plant where fruit = f.fruit_id) as amount,
           c.color_id,
           lc.value as color_name,
           c.hexadecimal,
           sum(hp.weight_in_grams) total_produced_in_grams
         from fruit f 
         inner join color c 
           on f.color = c.color_id
         left join localized_text_content lc 
           on c.color_id = lc.option_reference_id 
                           and lc.locale_id = 'de_DE'
                           and lc.label = 'FRUIT_COLOR'
         left join plant p
           on p.fruit = f.fruit_id
         left join harvest_plant hp
           on p.plant_id = hp.plant
         group by
           f.fruit_id,
           c.color_id,
           lc.value
        ;"
    )
    .fetch_all(pool.as_ref())
    .await
    .map(|rows| {
        rows.into_iter()
            .map(|row| Fruit {
                id: row.fruit_id,
                name: row.fruit_name,
                avg_weight_in_grams: row.avg_weight_in_grams.round(2),
                total_produced_in_grams: match row.total_produced_in_grams {
                    Some(big_d) => big_d,
                    None => BigDecimal::from(0),
                }
                .round(2),
                color: Color {
                    id: row.color_id,
                    name: row.color_name,
                    hexadecimal: row.hexadecimal,
                },
            })
            .collect()
    })
    .unwrap_or_else(|_| Vec::new());

    HttpResponse::Ok().json(fruits)
}

async fn print_plant_labels(pool: web::Data<PgPool>) -> impl Responder {
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

    return PrintPlantHtml { plants }.to_response();
}

#[derive(Template)]
#[template(path = "print_plant.html")]
pub struct PrintPlantHtml {
    plants: Vec<String>,
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
            .route("fruits", web::get().to(select_fruits))
            .route("plants/print-missing", web::get().to(print_plant_labels))
            // .route("plants", web::get().to(select_plants))
            // .route("harvests", web::post().to(select_harvest))
            .route("fruit", web::post().to(insert_fruit))
            .route("plant", web::post().to(insert_plant))
            .route("plant/dispose/{plant_id}", web::patch().to(dispose_plant))
            .route("harvest", web::post().to(insert_harvest))
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}
