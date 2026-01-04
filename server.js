import express from 'express';
import 'dotenv/config';

import redis from './config/redisConfig.js';
import transporter from "./config/mailTransporter.js";

import mailRoutes from './routes/router.mail.js';
import otpRoutes from './routes/router.otp.js';

const PORT = process.env.PORT || 3000;
const SECRET = process.env.SECRET;
const server = express();

server.use(express.json());
server.use(express.urlencoded({ extended: true }));

server.get("/", (req, res) => {
    return res.status(200).send(
        `
        <div style="height: 100%; width: 100%; 
            display: flex; align-items: center; justify-content: center;
            gap: 1rem; flex-direction: column; overflow: hidden">
            <h1 style="font-size: 80px; font-weight: bolder">Mailer</h1>
            <h3 style="color: green; position: absolute; top: 40%; right: 40%">Live</h3>
        </div>
        `
    );
})

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

server.use("/api/v1/mail", mailRoutes);
server.use("/api/v1/otp", otpRoutes);

(async () => {
    await redis.connect();
    server.listen(PORT, () => {
        console.log(`[Server] Listening on ${process.env.PORT}`);
    })
})();
