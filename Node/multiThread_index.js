const os = require('os');
const { performance } = require('perf_hooks');
const { Worker } = require('worker_threads');

async function handle_create_req(req, res) {

    const { plantId, startDate, endDate, interval } = req.body;
    const startTime = performance.now();

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

    const createduration = performance.now() - startTime;
    console.log(`Created the obj list in ${createduration} ms`);

    // Calculate CPU usage
    const cpuUsage = process.cpuUsage();
    const cpuUsageInPercentage = ((cpuUsage.user + cpuUsage.system) / (createduration * 1000)) * 100;

    // Calculate RAM usage
    const totalMemory = os.totalmem();
    const usedMemory = totalMemory - os.freemem();
    const ramUsageInBytes = usedMemory / (1024 * 1024); // convert to MB

    // Calculate Disk I/O
    // Add your logic to calculate Disk I/O here
    const diskIO = 0; // Placeholder for disk I/O

    const totalduration = performance.now() - startTime;
    console.log(`Total duration is ${totalduration} ms`);

    const insertduration = createduration;
    console.log(`Time for insertion into db -> ${insertduration} ms`);

    const programlang = "Node.js";
    res.status(201).json({
        data: "Data Added successfully",
        count: 0,
        createduration,
        insertduration,
        totalduration,
        programlang,
        cpuUsage: cpuUsageInPercentage.toFixed(2) + '%',
        ramUsage: ramUsageInBytes.toFixed(2) + ' MB',
        diskIO: diskIO.toFixed(2) + ' KB/s' // Replace 'diskIO' with actual Disk I/O value
    });
}

const http = require("http");
const express = require("express");
const app = express();
app.use(express.json());
app.post("/createdata", handle_create_req);
const server = http.createServer(app);
const port = process.env.PORT || 5000;
server.listen(port, () => {
    console.log(`Server listening on port ${port}`);
});