use crate::domain::{Color, Fruit};
use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use sqlx::types::BigDecimal;

pub async fn select_fruits(pool: web::Data<PgPool>) -> HttpResponse {
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
