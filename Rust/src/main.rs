use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use chrono::{DateTime, Utc};
use futures::stream::iter;
use rand::Rng;
use rayon::iter;
use sqlx::postgres::{PgConnectOptions, PgPool};
use sqlx::{Error, Executor, Postgres, Transaction};
use std::cell::RefCell;
use std::sync::Mutex;
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

async fn insert_data(pool: &PgPool, data: &[(i32, String, i32, i32)]) -> Result<usize, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let mut count = 0;

    for row in data {
        let createdat = &row.1; // createdat is a String
        let result: Result<_, sqlx::Error> = sqlx::query(
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
            return Err(result.unwrap_err());
        } else {
            count += 1;
        }
    }

    tx.commit().await?;
    Ok(count)
}

// new code
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
  
    let mut list = Vec::new();
    let mut rng = rand::thread_rng();
    let start_datetime = DateTime::parse_from_rfc3339(&start_date).unwrap().with_timezone(&Utc);
    let end_datetime = DateTime::parse_from_rfc3339(&end_date).unwrap().with_timezone(&Utc);
    let interval_duration = chrono::Duration::from_std(Duration::from_secs(interval.try_into().unwrap())).unwrap();

    let mut handles = Vec::new();
    for plant in plant_id {
        let pool_clone = pool.clone();
        let mut list_clone = list.clone();
        let handle = tokio::task::spawn(async move {
            let mut current_datetime = start_datetime;
            while current_datetime <= end_datetime {
                let obj = (
                    plant,
                    current_datetime.to_rfc3339(), // convert to string
                    fastrand::Rng::new().u32(1..=100),
                    fastrand::Rng::new().u32(1..=100),
                );
                list_clone.push(obj);
                current_datetime = current_datetime + interval_duration;
            }
            let mut tx = pool_clone.begin().await.unwrap();
            let count = insert_data(&pool_clone, &list_clone).await.unwrap();
            if let Err(_) = tx.commit().await {
                return 0;
            }
            count
        });
        handles.push(handle);
    }

    let mut count = 0;
    for handle in handles {
        count += handle.await.unwrap();
    }

    print!("Total duration is {} ns", start_time.elapsed().as_millis());
    let totalduration = start_time.elapsed().as_millis();
    
    let createduration = totalduration ;
    print!("Created the obj list in {} ns per plant", createduration);

    let insertduration = totalduration - createduration ;
    print!("Time for insertion into db -> {} ns per plant", insertduration);

    let programlang = format!("Rust");
    
    HttpResponse::Created().json(CreateDataResponse {
        data: "Data Added successfully".to_string(),
        count,
        createduration,
        insertduration,
        totalduration,
        programlang,
    })
}


// new code



//////// OLD code, keep safe

/*
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
  
    let mut list = Vec::new();
    let mut rng = rand::thread_rng();
    let start_datetime = DateTime::parse_from_rfc3339(&start_date).unwrap().with_timezone(&Utc);
    let end_datetime = DateTime::parse_from_rfc3339(&end_date).unwrap().with_timezone(&Utc);
    let interval_duration = chrono::Duration::from_std(Duration::from_secs(interval.try_into().unwrap())).unwrap();

    // This loop creates and stores data in a vector
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

    let createduration = start_time.elapsed().as_millis(); // stop timer
    print!("Created the obj list in {} ns", createduration);


    let mut tx = pool.begin().await.unwrap();
    let mut count = 0;

    // This loop takes data from the vector and stores it in the postgreSQL database
    /*
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

     */

//////  new start

// /* 
    let pool_clone = pool.clone();
    let insert_task = tokio::task::spawn_blocking(move || {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(insert_data(&pool_clone, &list))
    });


    let insert_result = insert_task.await.unwrap();
    count += insert_result.unwrap();
// */

///// new end

    print!("Total duration is {} ns", start_time.elapsed().as_millis());
    let totalduration = start_time.elapsed().as_millis();
    
    let insertduration = totalduration - createduration; // stop timer
    print!("Time for insertion into db -> {}ns", insertduration);

    let programlang = format!("Rust");
    if let Err(_) = tx.commit().await {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Created().json(CreateDataResponse {
        data: "Data Added successfully".to_string(),
        count,
        createduration,
        insertduration,
        totalduration,
        programlang,
    })  
}

*/

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
