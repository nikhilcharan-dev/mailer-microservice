import handlebars from "handlebars";

/* BASE */
const baseHead = `
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
</head>
`;

const baseFooter = `
<footer style="text-align:center; font-size:12px; color:#9ca3af; padding:16px;">
  © ${new Date().getFullYear()} {{appName}} · All rights reserved
</footer>
</html>
`;

/* OTP TEMPLATE */
const otpTemplate = handlebars.compile(`
${baseHead}
<style>
  body {
    margin: 0;
    padding: 24px 0;
    background: #f4f6f8;
    font-family: Arial, Helvetica, sans-serif;
  }
  .card {
    max-width: 600px;
    margin: auto;
    background: #ffffff;
    border-radius: 8px;
    overflow: hidden;
  }
  .header {
    background: #4f46e5;
    color: #ffffff;
    padding: 20px;
    text-align: center;
    font-size: 20px;
    font-weight: bold;
  }
  .content {
    padding: 32px 24px;
    font-size: 15px;
    line-height: 1.6;
    color: #333;
  }
  .otp {
    display: inline-block;
    background: #f3f4f6;
    padding: 14px 24px;
    font-size: 28px;
    letter-spacing: 6px;
    font-weight: bold;
    border-radius: 6px;
    margin: 24px 0;
  }
</style>

<body>
  <div class="card">
    <div class="header">Verify Your Email</div>
    <div class="content">
      <p>Hi <strong>{{name}}</strong>,</p>
      <p>Use the OTP below to complete your verification:</p>
      <div style="text-align:center;">
        <div class="otp">{{otp}}</div>
      </div>
      <p>This OTP is valid for a limited time.</p>
    </div>
  </div>
</body>
${baseFooter}
`);

/* GREETING TEMPLATE */
const greetingTemplate = handlebars.compile(`
${baseHead}
<style>
  body {
    margin: 0;
    padding: 24px 0;
    background: #f9fafb;
    font-family: Arial, Helvetica, sans-serif;
  }
  .card {
    max-width: 600px;
    margin: auto;
    background: #ffffff;
    border-radius: 8px;
    padding: 32px 24px;
  }
</style>

<body>
  <div class="card">
    <h2>Welcome Aboard 🎉</h2>
    <p>Hi <strong>{{name}}</strong>,</p>
    <p>
      Welcome to <strong>{{org}}</strong>!  
      We're excited to have you join as a <strong>{{role}}</strong>.
    </p>
  </div>
</body>
${baseFooter}
`);

/* NOTIFICATION TEMPLATE */
const notificationTemplate = handlebars.compile(`
${baseHead}
<style>
  body {
    margin: 0;
    padding: 24px;
    background: #ffffff;
    font-family: Arial, Helvetica, sans-serif;
  }
</style>

<body>
  <p>Hi <strong>{{name}}</strong>,</p>
  <p>This is a notification regarding recent activity on your account.</p>
  <p>No action is required.</p>
</body>
${baseFooter}
`);

/* FORGOT PASSWORD TEMPLATE */
const forgotPasswordTemplate = handlebars.compile(`
${baseHead}
<style>
  body {
    margin: 0;
    padding: 24px;
    background: #f4f6f8;
    font-family: Arial, Helvetica, sans-serif;
  }
  .btn {
    display: inline-block;
    padding: 12px 20px;
    background: #4f46e5;
    color: #ffffff;
    text-decoration: none;
    border-radius: 6px;
    font-weight: bold;
  }
</style>

<body>
  <h2>Password Reset</h2>
  <p>Hi <strong>{{name}}</strong>,</p>
  <p>Click the button below to reset your password:</p>
  <p>
    <a href="{{resetLink}}" class="btn">Reset Password</a>
  </p>
  <p>If you didn’t request this, you can ignore this email.</p>
</body>
${baseFooter}
`);

/* EXPORT HELPERS */
const getOtp = (email, otp, appName = "WorkPing") => {
    const name = email.split('@')[0].toUpperCase();
    return otpTemplate({ name, otp, appName })
}

const getGreeting = (name, org, role, appName = "WorkPing") =>
    greetingTemplate({ name, org, role, appName });

const getNotification = (name, appName = "WorkPing") =>
    notificationTemplate({ name, appName });

const getForgotPassword = (name, resetLink, appName = "WorkPing") =>
    forgotPasswordTemplate({ name, resetLink, appName });

export default {
    getOtp,
    getGreeting,
    getNotification,
    getForgotPassword
};