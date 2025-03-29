use actix_web::web::Data;
use actix_web::{App, HttpServer, web};
use dotenv::dotenv;
use pepper_manager::routes::{
    dispose_plant, insert_fruit, insert_harvest, insert_plant, print_plant_labels, select_fruits,
};
use serde::Serialize;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::collections::HashMap;
use std::env;

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let pool: PgPool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool.");

    let db_pool = Data::new(pool);

    HttpServer::new(move || {
        App::new()
            .service(
                web::scope("/admin")
                    .route("/plant", web::post().to(insert_plant))
                    .route("plants/print-missing", web::get().to(print_plant_labels))
                    .route("fruit", web::post().to(insert_fruit))
                    .route("fruits", web::get().to(select_fruits))
                    .route("plant/dispose/{plant_id}", web::delete().to(dispose_plant))
                    .route("harvest", web::post().to(insert_harvest)),
            )
            .app_data(db_pool.clone())
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}
