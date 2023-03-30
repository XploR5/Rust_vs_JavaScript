/*use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgConnectOptions, PgPool};

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
*/

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// New code

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use chrono::{DateTime, Utc};
use futures::stream::iter;
use rand::Rng;
use rayon::iter;
use sqlx::postgres::{PgConnectOptions, PgPool};
use sqlx::{Error, Executor, Postgres, Transaction};
use std::cell::RefCell;
use std::sync::Mutex;
use std::time::Duration;

#[derive(Debug, serde::Deserialize)]
struct CreateDataRequest {
    plant_id: Vec<i32>,
    start_date: String,
    end_date: String,
    interval: i32,
}

#[derive(Debug, serde::Serialize)]
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

/* 
    let mut list = vec![];

    for i in 0..plant_id.len() {
        let mut current_date = DateTime::parse_from_rfc3339(&start_date)
            .unwrap()
            .with_timezone(&Utc);
        let end_date = DateTime::parse_from_rfc3339(&end_date)
            .unwrap()
            .with_timezone(&Utc);

        while current_date <= end_date {
            let obj = (
                plant_id[i],
                current_date.to_rfc3339(), // convert to string
                rand::thread_rng().gen_range(1 .. 100),
                rand::thread_rng().gen_range(1 .. 100),
            );
            list.push(obj);
            current_date = current_date + chrono::Duration::seconds(interval as i64);
        }
    }

*/
    ////////////////// NEW ADDS ///////////////////
    
    let mut list = Vec::new();
    let mut rng = rand::thread_rng();
    let start_datetime = DateTime::parse_from_rfc3339(&start_date).unwrap().with_timezone(&Utc);
    let end_datetime = DateTime::parse_from_rfc3339(&end_date).unwrap().with_timezone(&Utc);
    let interval_duration = chrono::Duration::from_std(Duration::from_secs(interval.try_into().unwrap())).unwrap();

    for plant in plant_id {
    let mut current_datetime = start_datetime;
    while current_datetime <= end_datetime {
        let obj = (
            plant,
            current_datetime.to_rfc3339(), // convert to string
            rng.gen_range(1..100),
            rng.gen_range(1..100),
        );
        list.push(obj);
        current_datetime = current_datetime + interval_duration;
        }
    }

    ////////////////////////

    let mut tx = pool.begin().await.unwrap();
    let mut count = 0;

    for row in &list {
        let createdat = &row.1; // createdat is a String
        let result: Result<_, Error> = sqlx::query(
            "INSERT INTO single(plant_id, createdat, quality, performance) VALUES ($1, $2, $3, $4)",
        )
        .bind(&row.0)
        .bind(createdat)
        .bind(&row.2)
        .bind(&row.3)
        .execute(&mut tx)
        .await;

        if result.is_err() {
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().finish();
        } else {
            count += 1;
        }
    }

    if let Err(_) = tx.commit().await {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Created().json(CreateDataResponse {
        data: "Data Added successfully".to_string(),
        count,
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
