import { Router } from 'express';
import { sendEMail } from "../mail/google.mails.js";

const router = Router();

router.post("/send-mail", async (req, res) => {

});

export default router;