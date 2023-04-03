// use std::env;
use std::thread;
use futures::future::join_all;
use tokio::task;
// use rayon::iter;
// use std::sync::Arc;
// use async_std::task;
// use std::sync::Mutex;
// use rayon::prelude::*;
// use std::cell::RefCell;
// use futures::stream::iter;
// use tokio::runtime::Runtime;
// use sqlx::postgres::PgPoolOptions;
// use sqlx::{Executor, Postgres, Transaction};
use rand::SeedableRng;
use rand::rngs::StdRng;
use fastrand::i32;

use actix_web::{
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use chrono::{DateTime, Utc};
use postgres::Transaction;
use std::mem::replace;
use rand::Rng;
use sqlx::{postgres::{PgConnectOptions, PgPool}, pool, Postgres};
use sqlx::Error;
use std::time::{Duration, Instant};

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
    createduration: u128,
    insertduration: u128,
    totalduration: u128,
    programlang: String,
}


 async fn create_and_insert_data(plant: i32, start_datetime: DateTime<Utc>, end_datetime: DateTime<Utc>, interval_duration:  chrono::Duration, pool: web::Data<PgPool>){
    print!("plant: {}  wit st_dtc: {}  end: {} dur: {}\n", plant, start_datetime, end_datetime, interval_duration);
    
    let mut tx = pool.begin().await.unwrap(); // new
    // let mut rng = rand::thread_rng();
    let mut list = Vec::new();


    // This loop creates and stores data in a vector
    let mut current_datetime = start_datetime;
    while current_datetime <= end_datetime {
        let obj = (
            plant,
            current_datetime.to_rfc3339(), // convert to string
            i32(..=100),
            i32(..=100),
        );
        list.push(obj);
        current_datetime = current_datetime + interval_duration;
    }

    // This loop takes data from the vector with name list and stores it in the postgreSQL database
    for row in &list {
        let createdat = &row.1; // createdate is a String
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
            // let _ = &tx.rollback().await;
        }
    }
   let res = tx.commit().await;
    print!("The result is: {:#?}", res);
    // res

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

    let start_time = Instant::now(); // start timer
    let start_datetime = DateTime::parse_from_rfc3339(&start_date)
        .unwrap()
        .with_timezone(&Utc);
    let end_datetime = DateTime::parse_from_rfc3339(&end_date)
        .unwrap()
        .with_timezone(&Utc);
    let interval_duration =
        chrono::Duration::from_std(Duration::from_secs(interval.try_into().unwrap())).unwrap();


// // focus here 

    // for plant in plant_id {    
    //        create_and_insert_data(plant.clone(), start_datetime, end_datetime, interval_duration, pool.clone()).await;
    // }

    let mut handles = vec![];
    
    for plant in plant_id {
        let pool_clone = pool.clone();
        let handle = tokio::spawn(async move {
            create_and_insert_data(plant.clone(), start_datetime, end_datetime, interval_duration, pool_clone).await;
        });
        handles.push(handle);
    }

    // for plant in plant_id {    
    //     let cloned_plant = plant.clone();
    //     let cloned_pool = pool.clone();
    //     let cloned_start = start_datetime.clone();
    //     let cloned_end = end_datetime.clone();
    //     let cloned_duration = interval_duration.clone();
    
    //     task::spawn_blocking(move || {
    //         create_and_insert_data(cloned_plant, cloned_start, cloned_end, cloned_duration, cloned_pool)
    //     });
    // }


// // focus above




    // */
    let count = 0;

    let createduration = start_time.elapsed().as_millis(); // stop timer
    print!("Created the obj list in {} ns", createduration);

    print!("Total duration is {} ns", start_time.elapsed().as_millis());
    let totalduration = start_time.elapsed().as_millis();

    let insertduration = totalduration - createduration; // stop timer
    print!("Time for insertion into db -> {}ns", insertduration);

    let programlang = format!("Rust");

    // if let Err(_) = tx.commit().await {
    //     return HttpResponse::InternalServerError().finish();
    // }

    HttpResponse::Created().json(CreateDataResponse {
        data: "Data Added successfully".to_string(),
        count,
        createduration,
        insertduration,
        totalduration,
        programlang,
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
            .app_data(Data::new(pool.clone()))
            .service(web::resource("/createdata").route(web::post().to(create_data_for_plant)))
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}
