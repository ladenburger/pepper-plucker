use actix_web::HttpResponse;
use sqlx::{Postgres, Transaction};

pub async fn rollback_and_respond(transaction: Transaction<'_, Postgres>) -> HttpResponse {
    transaction.rollback().await.unwrap();

    return HttpResponse::InternalServerError().body(String::from(
        "Failed to create color: Database insert operation failed. ".to_owned(),
    ));
}
