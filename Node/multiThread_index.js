const { Pool } = require("pg");
const { DateTime, Duration } = require("luxon");


const BATCH_SIZE = 10000;

const dbPool = new Pool({
    host: "localhost",
    database: "testing_db",
    user: "postgres",
    password: "password",
});

async function createAndInsertData(plant, startDatetime, endDatetime, intervalDuration) {
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
    } catch (err) {
        await client.query("ROLLBACK");
        throw err;
    } finally {
        client.release();
    }
}

async function createDataForPlant(req, res) {
    const { plantId, startDate, endDate, interval } = req.body;

    const startTime = Date.now();
    const startDatetime = DateTime.fromISO(startDate, { zone: "utc" });
    const endDatetime = DateTime.fromISO(endDate, { zone: "utc" });
    const intervalDuration = Duration.fromObject({ seconds: interval });

    const promises = plantId.map((plant) => {
        return createAndInsertData(plant, startDatetime, endDatetime, intervalDuration).catch((err) => {
            console.error(`Error occurred: ${err}`);
        });
    });

    await Promise.all(promises);

    const count = 0;
    const createduration = Date.now() - startTime;
    console.log(`Created the obj list in ${createduration} ns`);
    const totalduration = Date.now() - startTime;
    console.log(`Total duration is ${totalduration} ns`);
    const insertduration = createduration;
    console.log(`Time for insertion into db -> ${insertduration} ns`);
    const programlang = "Node.js";
    res.status(201).json({
        data: "Data Added successfully",
        count,
        createduration,
        insertduration,
        totalduration,
        programlang,
    });
}

const http = require("http");
const express = require("express");
const app = express();

app.use(express.json());

app.post("/createdata", createDataForPlant);

const server = http.createServer(app);

const port = process.env.PORT || 5000;
server.listen(port, () => {
    console.log(`Server listening on port ${port}`);
});