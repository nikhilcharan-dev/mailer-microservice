import { Router } from "express";
import redis from "../config/redisConfig.js";
import { sendEmailOTP } from "../mail/mailer.js";

const router = Router();

const generatorOtp = (len) => {
    return Math.floor(Math.random() * (10 ** len)).toString();
}

router.post("/send-email-otp", async (req, res) => {
    const { email } = req.body;
    try {
        const otp = generatorOtp(6);

        await redis.set(`otp:email:${email}`, otp, { EX: 30 * 60 } );

        await sendEmailOTP(email, otp);

        return res.status(200).json({
            status: "success",
        });
    } catch (err) {
        console.log(err);
        return res.status(500).json({
            error: "Internal Server Error",
        })
    }
});

router.post("/send-phone-otp", async (req, res) => {
    const { phone } = req.body;
    try {
        if(!phone) {
            return res.status(400).json({
                error: "Phone number is required",
            })
        }
        const otp = generatorOtp(6);
        // await sendPhoneOTP(phone, otp);
        // await redis.set(`otp:phone:${phone}`, otp, { EX: 30 * 60 } );
        res.status(200).json({
            status: "success",
        })
    } catch (err) {
        console.log(err);
    }
});

router.post("/verify-email-otp", async (req, res) => {
    const { email, otp } = req.body;
    try {
        const OTP = await redis.get(`otp:email:${email}`);
        if(!OTP) return res.status(400).json({
            error: "Bad Request",
        })

        if(OTP !== otp) return res.status(400).json({
            return: "Invalid OTP"
        })

        await redis.del(`otp:email:${email}`);
        return res.status(200).json({
            verified: true,
            status: "success",
        })
    } catch (err) {
        console.log(err);
        res.status(500).json({
            error: "Internal Server Error",
        })
    }
});

router.post("/verify-phone-otp", async (req, res) => {
    const { phone, otp } = req.body;
    try {
        if(!phone || !otp) {
            return res.status(400).json({
                error: "Invalid Fields",
            })
        }

        if(phone.length !== 10) {
            return res.status(400).json({
                error: "Invalid Phone",
            })
        }

        // const OTP = await redis.get(`otp:phone:${phone}`);

        res.status(200).json({
            verified: true,
            status: "success",
        })
    } catch (err) {
        console.log(err);
        res.status(500).json({
            error: "Internal Server Error",
        })
    }
});

export default router;