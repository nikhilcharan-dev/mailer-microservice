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
server.use((req, res, next) => {
    const token = req.headers.authorization;
    if (!token || token !== SECRET) {
        return res.status(403).json({
            error: "unauthorized"
        })
    }
    const { email } = req.body;
    if(!email) {
        return res.status(403).json({
            error: "Body missing or invalid email"
        })
    }
    next();
})

server.use("/api/v1", mailRoutes);

(async () => {
    server.listen(PORT, () => {
        console.log(`Listening on ${process.env.PORT}`);
    })
    await redis.connect();
})();
