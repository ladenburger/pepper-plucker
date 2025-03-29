use crate::domain::DisposePlant;
use actix_web::{HttpResponse, web};
use chrono::Local;
use sqlx::PgPool;

pub async fn dispose_plant(
    pool: web::Data<PgPool>,
    path: web::Path<(String,)>,
    body: web::Json<DisposePlant>,
) -> HttpResponse {
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
