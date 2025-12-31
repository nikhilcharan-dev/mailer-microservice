import express from 'express';
import morgan from 'morgan';
import 'dotenv/config';

import redis from './config/redisConfig.js';
import transporter from "./mail/mailer.js";

import mailRoutes from './routes/router.js';

const server = express();
const PORT = process.env.PORT || 3000;
const SECRET = process.env.SECRET;

server.use(morgan('dev'));
server.use(express.json());
server.use(express.urlencoded({ extended: true }));

server.use("/api/v1", mailRoutes);

(async () => {
    server.listen(PORT, () => {
        console.log(`Listening on ${process.env.PORT}`);
    })
    await redis.connect();
})();
