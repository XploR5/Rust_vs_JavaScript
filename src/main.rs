use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgConnectOptions, PgPool};
use sqlx::types::time::OffsetDateTime;
use sqlx::{ConnectOptions, Executor};
use postgres::{NoTls};

#[derive(Debug, Deserialize)]
struct CreateDataRequest {
    plant_id: Vec<i32>,
    start_date: String,
    end_date: String,
    interval: i32,
}

#[derive(Debug, Serialize)]
struct CreateDataResponse {
    data: String,
    count: usize,
}

async fn create_data_for_plant(
    pool: web::Data<PgPool>,
    req: web::Json<CreateDataRequest>,
) -> impl Responder {
    let CreateDataRequest {
        plant_id,
        start_date,
        end_date,
        interval,
    } = req.into_inner();

    let mut list = vec![];

    for i in 0..plant_id.len() {
        let mut current_date = chrono::DateTime::parse_from_rfc3339(&start_date)
            .unwrap()
            .with_timezone(&chrono::Utc);
        let end_date = chrono::DateTime::parse_from_rfc3339(&end_date)
            .unwrap()
            .with_timezone(&chrono::Utc);

        while current_date <= end_date {
            let obj = (
                plant_id[i],
                current_date.to_rfc3339(), // convert to string
                rand::random::<i32>() % 100 + 1,
                rand::random::<i32>() % 100 + 1,
            );
            list.push(obj);
            current_date = current_date + chrono::Duration::seconds(interval as i64);
        }
    }

    for row in &list {
        let createdat = &row.1; // createdat is a String
        sqlx::query!(
            "INSERT INTO single(plant_id, createdat, quality, performance) VALUES ($1, $2, $3, $4)",
            &row.0, createdat, &row.2, &row.3
        )
        .execute(pool.as_ref())
        .await
        .unwrap();
    }

    HttpResponse::Created().json(CreateDataResponse {
        data: "Data Added successfully".to_string(),
        count: list.len(),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let connect_options = PgConnectOptions::new()
        .username("postgres")
        .password("password")
        .host("localhost")
        .database("testing_db");

    let pool = PgPool::connect_with(connect_options)
        .await
        .expect("Failed to create database pool");

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .service(web::resource("/createdata").route(web::post().to(create_data_for_plant)))
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}
