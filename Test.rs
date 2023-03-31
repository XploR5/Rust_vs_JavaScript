
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


// This code inserts the data into the database
    let mut tx = pool.begin().await.unwrap();
    let mut count = 0;
 
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