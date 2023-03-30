const express = require('express');
const { Pool } = require('pg');
const { DateTime } = require('luxon');
const bodyParser = require('body-parser');

const app = express();
const port = 5000;

app.use(bodyParser.json());

const pool = new Pool({
  user: 'postgres',
  password: 'password',
  host: 'localhost',
  database: 'testing_db',
});

app.post('/createdata', async (req, res) => {
  const { plantId, startDate, endDate, interval } = req.body;

  const programstart = Date.now();

  const list = [];

  for (let i = 0; i < plantId.length; i++) {
    let current_date = DateTime.fromISO(startDate, { zone: 'utc' });
    const end_date = DateTime.fromISO(endDate, { zone: 'utc' });

    while (current_date <= end_date) {
      const obj = {
        plant_id: plantId[i],
        createdat: current_date.toISO(),
        quality: Math.floor(Math.random() * 100) + 1,
        performance: Math.floor(Math.random() * 100) + 1,
      };
      list.push(obj);
      current_date = current_date.plus({ seconds: interval });
    }
  }

  const datacreate = Date.now() - programstart;

  const client = await pool.connect();

  try {
    await client.query('BEGIN');
    for (const row of list) {
      await client.query(
        'INSERT INTO single(plant_id, createdat, quality, performance) VALUES ($1, $2, $3, $4)',
        [row.plant_id, row.createdat, row.quality, row.performance]
      );
    }
    await client.query('COMMIT');
  
    const programend = Date.now() - programstart ;
    const datainsert = programend - datacreate;
    const programLang = "javascript";

    res.status(201).json({
      data: 'Data Added successfully',
      count: list.length,
      datacreate,
      datainsert,
      programend,
      programLang
    });
  } catch (e) {
    await client.query('ROLLBACK');
    throw e;
  } finally {
    client.release();
  }
});

app.listen(port, () => {
  console.log(`Server running at http://localhost:${port}`);
});
