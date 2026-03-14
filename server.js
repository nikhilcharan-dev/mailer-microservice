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
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Mailer Service | Status</title>
            <link rel="preconnect" href="https://fonts.googleapis.com">
            <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
            <link href="https://fonts.googleapis.com/css2?family=Plus+Jakarta+Sans:wght@400;600;800&display=swap" rel="stylesheet">
            <style>
                :root {
                    --bg: #f8fafc;
                    --text: #0f172a;
                    --text-muted: #64748b;
                    --accent: #2563eb;
                    --card: #ffffff;
                    --border: #e2e8f0;
                    --shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1);
                }
                @media (prefers-color-scheme: dark) {
                    :root {
                        --bg: #020617;
                        --text: #f8fafc;
                        --text-muted: #94a3b8;
                        --accent: #3b82f6;
                        --card: #0f172a;
                        --border: #1e293b;
                        --shadow: 0 4px 6px -1px rgb(0 0 0 / 0.5);
                    }
                }
                body {
                    margin: 0;
                    padding: 0;
                    height: 100vh;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    font-family: 'Plus Jakarta Sans', sans-serif;
                    background: var(--bg);
                    color: var(--text);
                    transition: background 0.3s ease;
                }
                .container {
                    text-align: center;
                    padding: 2.5rem;
                    background: var(--card);
                    border-radius: 24px;
                    border: 1px solid var(--border);
                    box-shadow: var(--shadow);
                    max-width: 400px;
                    width: 90%;
                    backdrop-filter: blur(8px);
                }
                h1 {
                    font-size: 3.5rem;
                    font-weight: 800;
                    margin: 0;
                    background: linear-gradient(135deg, var(--accent), #9333ea);
                    -webkit-background-clip: text;
                    -webkit-text-fill-color: transparent;
                }
                .status-badge {
                    display: inline-flex;
                    align-items: center;
                    gap: 0.5rem;
                    background: rgba(22, 163, 74, 0.1);
                    color: #16a34a;
                    padding: 6px 16px;
                    border-radius: 99px;
                    font-size: 0.875rem;
                    font-weight: 600;
                    margin: 1.5rem 0;
                }
                .status-dot {
                    width: 8px;
                    height: 8px;
                    background: #16a34a;
                    border-radius: 50%;
                    box-shadow: 0 0 12px #16a34a;
                    animation: pulse 2s infinite;
                }
                @keyframes pulse {
                    0% { transform: scale(0.95); opacity: 1; }
                    70% { transform: scale(1.5); opacity: 0; }
                    100% { transform: scale(0.95); opacity: 0; }
                }
                p {
                    color: var(--text-muted);
                    line-height: 1.6;
                    margin-bottom: 2rem;
                }
                .link {
                    color: var(--accent);
                    text-decoration: none;
                    font-weight: 600;
                    font-size: 0.95rem;
                    display: inline-flex;
                    align-items: center;
                    gap: 0.5rem;
                    transition: opacity 0.2s;
                }
                .link:hover {
                    opacity: 0.8;
                    text-decoration: underline;
                }
            </style>
        </head>
        <body>
            <div class="container">
                <h1>📧 Mailer</h1>
                <div class="status-badge">
                    <span class="status-dot"></span>
                    Service Operational
                </div>
                <p>Welcome to the WorkPing Mailer microservice. All systems are currently running within normal parameters.</p>
                <a href="/templates" class="link">Explore Email Templates →</a>
            </div>
        </body>
        </html>
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
            status: "error",
            error: "Unauthorized: Invalid or missing secret token"
        })
    }

    const { email } = req.body;
    if(!email) {
        return res.status(400).json({
            status: "error",
            error: "Bad Request: Recipient email is required"
        })
    }

    // Basic email format validation
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    if (!emailRegex.test(email)) {
        return res.status(400).json({
            status: "error",
            error: "Bad Request: Invalid email format"
        })
    }

    next();
});

server.use("/api/v1/mail", mailRoutes);
server.use("/api/v1/otp", otpRoutes);

(async () => {
    try {
        await redis.connect();
        server.listen(PORT, () => {
            console.log(`[Server] Listening on port ${PORT}`);
        });
    } catch (error) {
        console.error("[Server Initialization Error]", error);
        process.exit(1);
    }
})();
