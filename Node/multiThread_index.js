async function createDataForPlant(req, res) {
    const { plantId, startDate, endDate, interval } = req.body;
    const startTime = Date.now();

    const { Worker } = require('worker_threads');

    const createAndInsertDataThread = (plant, startDate, endDate, interval) => {
        return new Promise((resolve, reject) => {
            const worker = new Worker('./createAndInsertData.js', {
                workerData: { plant, startDate, endDate, interval }
            });
    
            worker.on('message', (result) => {
                resolve(result);
            });
    
            worker.on('error', (err) => {
                reject(err);
            });
            worker.postMessage('start');
        });
    };
    

    const promises = plantId.map(async (plant) => {
        try {
          await createAndInsertDataThread(plant, startDate, endDate, interval);
        } catch (err) {
          console.error(`Error occurred: ${err}`);
        }
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