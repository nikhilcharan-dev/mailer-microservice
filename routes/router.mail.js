import { Router } from 'express';
import { sendEMail } from "../mail/mailer.js";

const router = Router();

router.post("/send-mail", async (req, res) => {
    try {

    } catch(err) {
        console.log(err);
        res.status(500).send({
            success: false,
            error: "Internal Server Error"
        })
    }
});

export default router;