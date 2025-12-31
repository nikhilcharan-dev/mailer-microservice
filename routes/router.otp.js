import { Router } from "express";
import redis from "../config/redisConfig.js";
import {sendOTP} from "../mail/google.mails.js";

const router = Router();

const generatorOtp = (len) => {
    return Math.floor(Math.random() * (10 ** len)).toString();
}

router.post("/send-email-otp", async (req, res) => {
    try {
        const { email } = req.body;
        const otp = generatorOtp(6);

        await redis.set(`otp:${email}`, otp, { EX: 30 * 60 } );

        await sendOTP(email, otp);

        return res.status(200).json({
            status: "success",
        });
    } catch (err) {
        console.log(err);
    }
});

router.post("/send-phone-otp", async (req, res) => {
    try {

    } catch (err) {
        console.log(err);
    }
});

router.post("/verify-email-otp", async (req, res) => {
    try {
        const { email, otp } = req.body;
        const OTP = await redis.get(`otp:${email}`);
        if(!OTP) return res.status(404).send({
            error: "OTP expired"
        })

        if(OTP !== otp) return res.status(404).send({
            return: "Invalid OTP"
        })

        await redis.del(`otp:${email}`);
        return res.status(200).json({
            status: "success",
        })
    } catch (err) {
        console.log(err);
    }
});

router.post("/verify-phone-otp", async (req, res) => {
    try {

    } catch (err) {
        console.log(err);
    }
});

export default router;