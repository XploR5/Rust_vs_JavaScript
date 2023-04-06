const { parentPort, workerData } = require('worker_threads');
const { Pool } = require('pg');
const { DateTime, Duration } = require("luxon");

const BATCH_SIZE = 10000;

const dbPool = new Pool({
    host: "localhost",
    database: "testing_db",
    user: "postgres",
    password: "password",
});


async function createAndInsertData(workerData) {
    const { plant, startDate, endDate, interval } = workerData;
        
    const startDatetime = DateTime.fromISO(startDate, { zone: "utc" });
    const endDatetime = DateTime.fromISO(endDate, { zone: "utc" });
    const intervalDuration = Duration.fromObject({ seconds: interval });
    
    console.log(`plant: ${plant} with st_dtc: ${startDatetime}  end: ${endDatetime} dur: ${intervalDuration}`);


    const list = [];

    let currentDatetime = startDatetime;
    while (currentDatetime <= endDatetime) {
        const obj = [plant, currentDatetime.toISO(), Math.floor(Math.random() * 100), Math.floor(Math.random() * 100)];
        list.push(obj);
        currentDatetime = currentDatetime.plus(intervalDuration);
    }

    const client = await dbPool.connect();
    try {
        await client.query("BEGIN");

        let buffer = [];
        for (let i = 0; i < list.length; i++) {
            const row = list[i];
            const createdat = row[1];

            buffer.push(row);
            if (buffer.length >= BATCH_SIZE || i == list.length - 1) {
                const valueStrings = buffer
                    .map((_, i) => `($${i * 4 + 1}, $${i * 4 + 2}, $${i * 4 + 3}, $${i * 4 + 4})`)
                    .join(", ");

                const queryStr = `INSERT INTO single(plant_id, createdat, quality, performance) VALUES ${valueStrings}`;

                const values = buffer.flat();

                await client.query(queryStr, values);

                buffer = [];
            }
        }

        await client.query("COMMIT");
        console.log("The result is: committed");

        // send the result back to the parent thread
        parentPort.postMessage('done');
    } catch (err) {
        await client.query("ROLLBACK");
        throw err;
    } finally {
        client.release();
    }
}

// listen for the 'message' event and call the createAndInsertData function
parentPort.on('message', (msg) => {
    if (msg === 'start') {
    console.log(workerData.plant)
        createAndInsertData(workerData);
    }
});
