import transporter from "./mailer.js";

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

export const sendOTP = async (email, otp) => {
    try {
        await transporter.sendMail({
            to: email,
            subject: "Verification One-Time-Password",
            html: `
                <h1> ${otp} is your verification code.</h1>
            `
        })
    } catch (err) {
        console.log("Node Mail Error: ", err.message);
        return Promise.reject(err);
    }
}