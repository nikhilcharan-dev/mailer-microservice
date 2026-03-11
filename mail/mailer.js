import transporter from "../config/mailTransporter.js";
import templates from "./templates.js";

/* ─── Generic: send raw HTML ─── */
export const sendEMail = async (email, subject, content) => {
    try {
        await transporter.sendMail({
            to: email,
            subject: subject,
            html: content,
        });
    } catch (err) {
        console.log("Node Mail Error: ", err.message);
        return Promise.reject(err);
    }
};

/* ─── OTP: Email Verification ─── */
export const sendEmailOTP = async (email, otp) => {
    try {
        await transporter.sendMail({
            to: email,
            subject: "OTP Verification",
            html: templates.getOtp(email, otp),
        });
    } catch (err) {
        console.log("Node Mail Error: ", err.message);
        return Promise.reject(err);
    }
};

/* ─── OTP: Reset Password ─── */
export const sendResetPasswordOTP = async (email, otp) => {
    try {
        await transporter.sendMail({
            to: email,
            subject: "Reset Password OTP",
            html: templates.getResetPasswordOtp(email, otp),
        });
    } catch (err) {
        console.log("Node Mail Error: ", err.message);
        return Promise.reject(err);
    }
};

/* ─── Confirm: Password Verified ─── */
export const sendVerifyPassword = async (email) => {
    try {
        await transporter.sendMail({
            to: email,
            subject: "Password Verified",
            html: templates.getVerifyPassword(email),
        });
    } catch (err) {
        console.log("Node Mail Error: ", err.message);
        return Promise.reject(err);
    }
};

/* ─── Forgot Password (link-based) ─── */
export const sendForgotPassword = async (email, resetLink) => {
    try {
        await transporter.sendMail({
            to: email,
            subject: "Reset Your Password",
            html: templates.getForgotPassword(email, resetLink),
        });
    } catch (err) {
        console.log("Node Mail Error: ", err.message);
        return Promise.reject(err);
    }
};

/* ─── Welcome / Greeting ─── */
export const sendGreeting = async (email, name, org, role) => {
    try {
        await transporter.sendMail({
            to: email,
            subject: `Welcome to ${org}!`,
            html: templates.getGreeting(name, org, role),
        });
    } catch (err) {
        console.log("Node Mail Error: ", err.message);
        return Promise.reject(err);
    }
};

/* ─── Alert: Info ─── */
export const sendAlertInfo = async (email, title, message) => {
    try {
        await transporter.sendMail({
            to: email,
            subject: title,
            html: templates.getAlertInfo(email, title, message),
        });
    } catch (err) {
        console.log("Node Mail Error: ", err.message);
        return Promise.reject(err);
    }
};

/* ─── Alert: Warning ─── */
export const sendAlertWarning = async (email, title, message, actionLink) => {
    try {
        await transporter.sendMail({
            to: email,
            subject: `⚠️ ${title}`,
            html: templates.getAlertWarning(email, title, message, actionLink),
        });
    } catch (err) {
        console.log("Node Mail Error: ", err.message);
        return Promise.reject(err);
    }
};

/* ─── Alert: Danger ─── */
export const sendAlertDanger = async (email, title, message, actionLink) => {
    try {
        await transporter.sendMail({
            to: email,
            subject: `🚨 ${title}`,
            html: templates.getAlertDanger(email, title, message, actionLink),
        });
    } catch (err) {
        console.log("Node Mail Error: ", err.message);
        return Promise.reject(err);
    }
};

/* ─── Alert: Success ─── */
export const sendAlertSuccess = async (email, title, message) => {
    try {
        await transporter.sendMail({
            to: email,
            subject: `✅ ${title}`,
            html: templates.getAlertSuccess(email, title, message),
        });
    } catch (err) {
        console.log("Node Mail Error: ", err.message);
        return Promise.reject(err);
    }
};

/* ─── Notification (Generic) ─── */
export const sendNotification = async (email, title, message) => {
    try {
        await transporter.sendMail({
            to: email,
            subject: title,
            html: templates.getNotification(email, title, message),
        });
    } catch (err) {
        console.log("Node Mail Error: ", err.message);
        return Promise.reject(err);
    }
};

/* ─── Raw HTML (just send pre-built HTML to an address) ─── */
export const sendRawHTML = async (email, subject, html) => {
    try {
        await transporter.sendMail({
            to: email,
            subject: subject,
            html: html,
        });
    } catch (err) {
        console.log("Node Mail Error: ", err.message);
        return Promise.reject(err);
    }
};
