use actix_web::{
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use chrono::{DateTime, Utc};
use fastrand::i32;
use sqlx::postgres::{PgConnectOptions, PgPool};
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

async fn create_and_insert_data(
    plant: i32,
    start_datetime: DateTime<Utc>,
    end_datetime: DateTime<Utc>,
    interval_duration: chrono::Duration,
    pool: web::Data<PgPool>,
) -> Result<(), sqlx::Error> {
    print!(
        "plant: {} with st_dtc: {}  end: {} dur: {}\n",
        plant, start_datetime, end_datetime, interval_duration
    );

    let mut list = Vec::new();

    let mut current_datetime = start_datetime;

    while current_datetime <= end_datetime {
        let obj = (
            plant,
            current_datetime.to_rfc3339(),
            i32(..=100),
            i32(..=100),
        );
        list.push(obj);
        current_datetime = current_datetime + interval_duration;
    }

// ------------------ //

let mut tx = pool.begin().await?;

// Define the size of the batch
const BATCH_SIZE: usize = 10000;

// Create a buffer to hold the rows
let mut buffer = Vec::with_capacity(BATCH_SIZE);

for (index, row) in list.iter().enumerate() {
    let createdat = &row.1;

    // Add the row to the buffer
    buffer.push((row.0, createdat, row.2, row.3));

    // If the buffer is full or we have reached the end of the list
    if buffer.len() >= BATCH_SIZE || index == list.len() - 1 {
        // Generate the query string with placeholders for multiple rows
        let query_str = format!(
            "INSERT INTO single(plant_id, createdat, quality, performance) VALUES {}",
            buffer
                .iter()
                .enumerate()
                .map(|(i, _)| format!("(${}, ${}, ${}, ${})", i * 4 + 1, i * 4 + 2, i * 4 + 3, i * 4 + 4))
                .collect::<Vec<_>>()
                .join(", ")
        );

        // Execute the insert statement with all the rows in the buffer
        let mut query = sqlx::query(&query_str);
        for (i, row) in buffer.iter().enumerate() {
            query = query.bind(row.0).bind(row.1).bind(row.2).bind(row.3);
        }
        let result = query.execute(&mut tx).await;

        // Clear the buffer
        buffer.clear();

        if result.is_err() {
            let _ = &tx.rollback().await;
            return Err(result.unwrap_err());
        }
    }
}

tx.commit().await?;

    

    // --------------------- //

    print!("The result is: committed\n");
    Ok(())
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

    let mut handles = vec![];

    for plant in plant_id {
        let pool_clone = pool.clone();
        let handle = tokio::spawn(async move {
            match create_and_insert_data(
                plant.clone(),
                start_datetime,
                end_datetime,
                interval_duration,
                pool_clone,
            )
            .await
            {
                Ok(res) => {
                    print!("The result is: {:#?}", res);
                    res
                }
                Err(e) => {
                    eprintln!("Error occurred: {}", e);
                    // Err(e)
                }
            }
        });
        handles.push(handle);
    }

    let results = futures::future::join_all(handles).await;
    for res in results {
        if let Err(e) = res {
            eprintln!("Error occurred: {}", e);
        }
    }

    let count = 0;
    let createduration = start_time.elapsed().as_millis(); // stop timer
    print!("Created the obj list in {} ns", createduration);
    print!("Total duration is {} ns", start_time.elapsed().as_millis());
    let totalduration = start_time.elapsed().as_millis();
    let insertduration = createduration;
    print!("Time for insertion into db -> {}ns", insertduration);
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
