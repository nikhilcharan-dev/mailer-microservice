import transporter from "../config/mailTransporter.js";
import templates from "./templates.js";

export const sendEMail = async (email, subject, content) => {
    try {
        await transporter.sendMail({
            to: email,
            subject: subject,
            html: content
        })
    } catch (err) {
        console.log("Node Mail Error: ", err.message);
        return Promise.reject(err);
    }
}

export const sendEmailOTP = async (email, otp) => {
    try {
        await transporter.sendMail({
            to: email,
            subject: "OTP Verification",
            html: templates.getOtp(email, otp)
        })
    } catch (err) {
        console.log("Node Mail Error: ", err.message);
        return Promise.reject(err);
    }
}