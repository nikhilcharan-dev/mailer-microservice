import express from 'express';
import 'dotenv/config';
import path from 'path';
import { fileURLToPath } from 'url';

import redis from './config/redisConfig.js';
import transporter from "./config/mailTransporter.js";

import mailRoutes from './routes/router.mail.js';
import otpRoutes from './routes/router.otp.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const PORT = process.env.PORT || 3000;
const SECRET = process.env.SECRET;
const server = express();

server.use(express.json());
server.use(express.urlencoded({ extended: true }));

/* ─── Public: Landing Page ─── */
server.get("/", (req, res) => {
    return res.status(200).send(
        `
        <div style="height: 100vh; width: 100%; 
            display: flex; align-items: center; justify-content: center;
            gap: 1rem; flex-direction: column; overflow: hidden;
            font-family: 'Segoe UI', Roboto, sans-serif; background: #f8fafc;">
            <h1 style="font-size: 64px; font-weight: 800; color: #1e293b; margin: 0;">📧 Mailer</h1>
            <span style="background: #16a34a; color: white; padding: 4px 16px; border-radius: 20px; font-size: 14px; font-weight: 600;">● Live</span>
            <p style="color: #64748b; margin-top: 16px;">
                <a href="/templates" style="color: #2563eb; text-decoration: none; font-weight: 600;">View Email Templates →</a>
            </p>
        </div>
        `
    );
});

/* ─── Public: Template Preview Page ─── */
server.get("/templates", (req, res) => {
    return res.sendFile(path.join(__dirname, "public", "templates.html"));
});

/* ─── Auth Middleware (protects API routes) ─── */
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
