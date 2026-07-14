// ─── Email Service ─────────────────────────────────────────────────────────────
//
// Provides a production SMTP email sender with a development fallback.
//
// Strategy:
//   Production  → SMTP via lettre (TLS, authenticated)
//   Development → tracing::info! stdout logging only (no network traffic)
//
// The correct mode is determined at construction time by inspecting the
// `AppConfig::smtp_configured()` helper — all four of SMTP_HOST, SMTP_PORT,
// SMTP_USERNAME, and SMTP_PASSWORD must be present to enter production mode.
//
// References:
//   https://docs.rs/lettre
// ────────────────────────────────────────────────────────────────────────────────

use lettre::{
    message::{header::ContentType, Mailbox},
    transport::smtp::authentication::Credentials,
    Address, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

use crate::config::AppConfig;

#[derive(Debug, Clone, serde::Serialize)]
pub struct EmailDiagnostics {
    pub configured: bool,
    pub connected: bool,
    pub auth_status: String, // "Success", "Failed", "Unknown"
    pub last_delivery: Option<chrono::DateTime<chrono::Utc>>,
    pub last_error: Option<String>,
}

/// Operational mode used by [`EmailService`].
enum DeliveryMode {
    /// Live SMTP relay — sends real emails.
    Smtp {
        transport: Box<AsyncSmtpTransport<Tokio1Executor>>,
        from_mailbox: Mailbox,
    },
    /// Development fallback — prints to stdout only.
    DevLog { from_email: String },
}

/// Application-scoped email service.
/// Clone-safe via the inner `Arc` on the transport.
pub struct EmailService {
    mode: DeliveryMode,
    from_name: String,
    diagnostics: std::sync::Arc<std::sync::RwLock<EmailDiagnostics>>,
}

impl EmailService {
    /// Construct from application config.
    ///
    /// If SMTP is fully configured, returns a live SMTP sender.
    /// Otherwise returns the dev-log fallback and emits a startup warning.
    pub fn from_config(config: &AppConfig) -> Self {
        if config.smtp_configured() {
            // SAFETY: smtp_host, port, username, password guaranteed Some by smtp_configured() check
            let host = config.smtp_host.as_ref().unwrap();
            let port = config.smtp_port.unwrap();
            let username = config.smtp_username.as_ref().unwrap();
            let password = config.smtp_password.as_ref().unwrap();

            let creds = Credentials::new(username.clone(), password.clone());

            let app_env = &config.app_environment;
            let is_prod = app_env.to_lowercase() == "production";

            let tls_option = if is_prod {
                let tls_mode = config.smtp_tls_mode.as_ref();
                let tls_mode_str = tls_mode.map(|s| s.to_lowercase());

                match tls_mode_str.as_deref() {
                    Some("none") => {
                        panic!("\n[FATAL] SMTP_TLS_MODE=none is insecure and is rejected in production mode.\n");
                    }
                    Some("implicit") => lettre::transport::smtp::client::Tls::Wrapper(
                        lettre::transport::smtp::client::TlsParameters::new(host.clone())
                            .unwrap_or_else(|e| {
                                panic!("\n[FATAL] Invalid SMTP TLS parameters: {}\n", e)
                            }),
                    ),
                    Some("starttls") => lettre::transport::smtp::client::Tls::Required(
                        lettre::transport::smtp::client::TlsParameters::new(host.clone())
                            .unwrap_or_else(|e| {
                                panic!("\n[FATAL] Invalid SMTP TLS parameters: {}\n", e)
                            }),
                    ),
                    Some(other) => {
                        panic!("\n[FATAL] Invalid SMTP_TLS_MODE '{}'. Supported modes are 'implicit', 'starttls'.\n", other);
                    }
                    None => {
                        // Default based on port
                        if port == 465 {
                            lettre::transport::smtp::client::Tls::Wrapper(
                                lettre::transport::smtp::client::TlsParameters::new(host.clone())
                                    .unwrap_or_else(|e| {
                                        panic!("\n[FATAL] Invalid SMTP TLS parameters: {}\n", e)
                                    }),
                            )
                        } else {
                            lettre::transport::smtp::client::Tls::Required(
                                lettre::transport::smtp::client::TlsParameters::new(host.clone())
                                    .unwrap_or_else(|e| {
                                        panic!("\n[FATAL] Invalid SMTP TLS parameters: {}\n", e)
                                    }),
                            )
                        }
                    }
                }
            } else {
                let tls_mode = config.smtp_tls_mode.as_ref();
                let tls_mode_str = tls_mode.map(|s| s.to_lowercase());
                match tls_mode_str.as_deref() {
                    Some("implicit") => lettre::transport::smtp::client::Tls::Wrapper(
                        lettre::transport::smtp::client::TlsParameters::new(host.clone())
                            .unwrap_or_else(|e| {
                                panic!("\n[FATAL] Invalid SMTP TLS parameters: {}\n", e)
                            }),
                    ),
                    Some("starttls") => lettre::transport::smtp::client::Tls::Required(
                        lettre::transport::smtp::client::TlsParameters::new(host.clone())
                            .unwrap_or_else(|e| {
                                panic!("\n[FATAL] Invalid SMTP TLS parameters: {}\n", e)
                            }),
                    ),
                    _ => lettre::transport::smtp::client::Tls::None,
                }
            };

            let transport = AsyncSmtpTransport::<Tokio1Executor>::relay(host)
                .unwrap_or_else(|e| {
                    panic!(
                        "\n[FATAL] Cannot build SMTP transport for host '{}': {}\n",
                        host, e
                    )
                })
                .port(port)
                .credentials(creds)
                .tls(tls_option)
                .timeout(Some(std::time::Duration::from_secs(
                    config.smtp_timeout_seconds,
                )))
                .build();

            let from_address: Address = config.smtp_from_email.parse().unwrap_or_else(|e| {
                panic!(
                    "\n[FATAL] SMTP_FROM_EMAIL '{}' is not a valid email address: {}\n",
                    config.smtp_from_email, e
                )
            });

            let from_mailbox = Mailbox::new(Some(config.smtp_from_name.clone()), from_address);

            tracing::info!(
                "Email service initialised in PRODUCTION mode (SMTP relay: {}:{})",
                host,
                port
            );

            let diagnostics = std::sync::Arc::new(std::sync::RwLock::new(EmailDiagnostics {
                configured: true,
                connected: false,
                auth_status: "Unknown".to_string(),
                last_delivery: None,
                last_error: None,
            }));

            Self {
                mode: DeliveryMode::Smtp {
                    transport: Box::new(transport),
                    from_mailbox,
                },
                from_name: config.smtp_from_name.clone(),
                diagnostics,
            }
        } else {
            tracing::warn!(
                "SMTP not configured (SMTP_HOST / SMTP_PORT / SMTP_USERNAME / SMTP_PASSWORD \
                 missing). Email service running in DEVELOPMENT logging-only mode. \
                 Password recovery emails will NOT be delivered."
            );

            let diagnostics = std::sync::Arc::new(std::sync::RwLock::new(EmailDiagnostics {
                configured: false,
                connected: false,
                auth_status: "Unknown".to_string(),
                last_delivery: None,
                last_error: None,
            }));

            Self {
                mode: DeliveryMode::DevLog {
                    from_email: config.smtp_from_email.clone(),
                },
                from_name: config.smtp_from_name.clone(),
                diagnostics,
            }
        }
    }

    pub fn get_diagnostics(&self) -> EmailDiagnostics {
        // SAFETY: RwLock read, only fails if poisoned
        self.diagnostics.read().unwrap().clone()
    }

    fn update_diagnostics_success(&self) {
        if let Ok(mut diag) = self.diagnostics.write() {
            diag.connected = true;
            diag.auth_status = "Success".to_string();
            diag.last_delivery = Some(chrono::Utc::now());
            diag.last_error = None;
        }
    }

    fn update_diagnostics_failure(&self, err: &str) {
        if let Ok(mut diag) = self.diagnostics.write() {
            diag.last_error = Some(err.to_string());
            let err_lower = err.to_lowercase();
            if err_lower.contains("authentication failed") || err_lower.contains("535") {
                diag.connected = true;
                diag.auth_status = "Failed".to_string();
            } else {
                diag.connected = false;
                diag.auth_status = "Unknown".to_string();
            }
        }
    }

    /// Send a password-recovery email.
    ///
    /// # Arguments
    /// * `to_email`   — Recipient email address (already validated by the caller)
    /// * `reset_link` — The full, production-ready reset URL containing the token
    ///
    /// # Security contract
    /// This method MUST NOT log `reset_link` when running in production mode.
    /// The plaintext token embedded in the link is single-use and sensitive.
    /// Dev mode logs the link to stdout deliberately for local testing.
    pub async fn send_password_reset(
        &self,
        to_email: &str,
        reset_link: &str,
    ) -> Result<(), String> {
        let subject = format!("{} — Password Reset Request", self.from_name);
        let body_html = build_reset_email_html(&self.from_name, reset_link);
        let body_plain = format!(
            "You requested a password reset for your {} account.\n\n\
             Use the following link to set a new password (expires in 1 hour):\n\
             {}\n\n\
             If you did not request this, you can safely ignore this email.\n\
             Your password will NOT change unless you follow the link above.\n\n\
             — {} Team",
            self.from_name, reset_link, self.from_name
        );

        match &self.mode {
            DeliveryMode::Smtp {
                transport,
                from_mailbox,
            } => {
                let to_address: Address = to_email
                    .parse()
                    .map_err(|e| format!("Invalid recipient address '{}': {}", to_email, e))?;
                let to_mailbox = Mailbox::new(None, to_address);

                let email = Message::builder()
                    .from(from_mailbox.clone())
                    .to(to_mailbox)
                    .subject(&subject)
                    .multipart(
                        lettre::message::MultiPart::alternative()
                            .singlepart(
                                lettre::message::SinglePart::builder()
                                    .header(ContentType::TEXT_PLAIN)
                                    .body(body_plain),
                            )
                            .singlepart(
                                lettre::message::SinglePart::builder()
                                    .header(ContentType::TEXT_HTML)
                                    .body(body_html),
                            ),
                    )
                    .map_err(|e| format!("Failed to build email message: {}", e))?;

                let send_res = transport.send(email).await;
                if let Err(ref e) = send_res {
                    let err_msg = format!("SMTP delivery error: {}", e);
                    self.update_diagnostics_failure(&err_msg);
                    return Err(err_msg);
                }
                self.update_diagnostics_success();

                // ⚠  Do NOT log reset_link in production — token is sensitive
                tracing::info!(
                    target: "security",
                    recipient = %to_email,
                    "Password reset email dispatched via SMTP"
                );

                Ok(())
            }

            DeliveryMode::DevLog { from_email } => {
                // Safe to log reset_link in development — no real user data
                tracing::info!(
                    target: "security",
                    "\n==============================================================\n\
                     [DEV EMAIL — NOT SENT] Password Reset\n\
                     --------------------------------------------------------------\n\
                     From : {} <{}>\n\
                     To   : {}\n\
                     Subj : {}\n\
                     Link : {}\n\
                     Exp  : 1 hour\n\
                     ==============================================================",
                    self.from_name,
                    from_email,
                    to_email,
                    subject,
                    reset_link,
                );

                self.update_diagnostics_success();
                Ok(())
            }
        }
    }

    /// Send an inquiry notification email to a vendor.
    #[allow(clippy::too_many_arguments)]
    pub async fn send_inquiry_notification(
        &self,
        to_email: &str,
        customer_name: &str,
        customer_phone: &str,
        customer_email: &str,
        event_date: &str,
        guest_count: i32,
        message: &str,
        listing_title: &str,
    ) -> Result<(), String> {
        let subject = format!("{} — New Client Inquiry Received", self.from_name);

        let body_plain = format!(
            "You have received a new inquiry on {}:\n\n\
             Customer Details:\n\
             - Name: {}\n\
             - Phone: {}\n\
             - Email: {}\n\n\
             Event Details:\n\
             - Listing: {}\n\
             - Event Date: {}\n\
             - Guest Count: {}\n\n\
             Message:\n\
             {}\n\n\
             Login to your Vendor Portal to respond.",
            self.from_name,
            customer_name,
            customer_phone,
            customer_email,
            listing_title,
            event_date,
            guest_count,
            message
        );

        let body_html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>New Inquiry Received</title>
</head>
<body style="margin:0;padding:0;background:#f4f4f4;font-family:Arial,sans-serif;">
  <table width="100%" cellpadding="0" cellspacing="0" style="background:#f4f4f4;padding:40px 0;">
    <tr>
      <td align="center">
        <table width="560" cellpadding="0" cellspacing="0" style="background:#fff;border-radius:8px;overflow:hidden;box-shadow:0 2px 8px rgba(0,0,0,0.08);">
          <tr>
            <td style="background:#1a1a2e;padding:32px 40px;text-align:center;">
              <h1 style="color:#fff;margin:0;font-size:22px;">{brand}</h1>
            </td>
          </tr>
          <tr>
            <td style="padding:40px;">
              <h2 style="color:#1a1a2e;font-size:20px;margin:0 0 16px;">New Inquiry Received</h2>
              <p style="color:#555;line-height:1.6;margin:0 0 24px;">
                You have received a new lead inquiry on your product page.
              </p>
              
              <h3 style="color:#1a1a2e;font-size:16px;margin:24px 0 8px;border-bottom:1px solid #eee;padding-bottom:8px;">Customer Information</h3>
              <table width="100%" cellpadding="4" cellspacing="0" style="color:#555;font-size:14px;line-height:1.5;">
                <tr><td width="30%"><strong>Name:</strong></td><td>{name}</td></tr>
                <tr><td><strong>Phone:</strong></td><td>{phone}</td></tr>
                <tr><td><strong>Email:</strong></td><td>{email}</td></tr>
              </table>

              <h3 style="color:#1a1a2e;font-size:16px;margin:24px 0 8px;border-bottom:1px solid #eee;padding-bottom:8px;">Event Details</h3>
              <table width="100%" cellpadding="4" cellspacing="0" style="color:#555;font-size:14px;line-height:1.5;">
                <tr><td width="30%"><strong>Listing:</strong></td><td>{title}</td></tr>
                <tr><td><strong>Date:</strong></td><td>{date}</td></tr>
                <tr><td><strong>Guest Count:</strong></td><td>{guests}</td></tr>
              </table>

              <h3 style="color:#1a1a2e;font-size:16px;margin:24px 0 8px;border-bottom:1px solid #eee;padding-bottom:8px;">Message</h3>
              <div style="background:#f9f9f9;padding:16px;border-radius:6px;color:#555;font-size:14px;line-height:1.5;font-style:italic;">
                {msg}
              </div>
            </td>
          </tr>
          <tr>
            <td style="background:#f9f9f9;padding:20px 40px;border-top:1px solid #eee;text-align:center;">
              <p style="color:#bbb;font-size:12px;margin:0;">
                © 2026 {brand}. All rights reserved.
              </p>
            </td>
          </tr>
        </table>
      </td>
    </tr>
  </table>
</body>
</html>"#,
            brand = self.from_name,
            name = customer_name,
            phone = customer_phone,
            email = customer_email,
            title = listing_title,
            date = event_date,
            guests = guest_count,
            msg = message
        );

        match &self.mode {
            DeliveryMode::Smtp {
                transport,
                from_mailbox,
            } => {
                let to_address: Address = to_email
                    .parse()
                    .map_err(|e| format!("Invalid recipient address '{}': {}", to_email, e))?;
                let to_mailbox = Mailbox::new(None, to_address);

                let email = Message::builder()
                    .from(from_mailbox.clone())
                    .to(to_mailbox)
                    .subject(&subject)
                    .multipart(
                        lettre::message::MultiPart::alternative()
                            .singlepart(
                                lettre::message::SinglePart::builder()
                                    .header(ContentType::TEXT_PLAIN)
                                    .body(body_plain),
                            )
                            .singlepart(
                                lettre::message::SinglePart::builder()
                                    .header(ContentType::TEXT_HTML)
                                    .body(body_html),
                            ),
                    )
                    .map_err(|e| format!("Failed to build email message: {}", e))?;

                let send_res = transport.send(email).await;
                if let Err(ref e) = send_res {
                    let err_msg = format!("SMTP delivery error: {}", e);
                    self.update_diagnostics_failure(&err_msg);
                    return Err(err_msg);
                }
                self.update_diagnostics_success();

                tracing::info!(
                    recipient = %to_email,
                    "Inquiry notification email dispatched via SMTP"
                );

                Ok(())
            }

            DeliveryMode::DevLog { from_email } => {
                tracing::info!(
                    "\n==============================================================\n\
                     [DEV EMAIL — NOT SENT] New Inquiry Notification\n\
                     --------------------------------------------------------------\n\
                     From : {} <{}>\n\
                     To   : {}\n\
                     Subj : {}\n\
                     Cust : {} ({})\n\
                     List : {}\n\
                     Date : {}\n\
                     Msg  : {}\n\
                     ==============================================================",
                    self.from_name,
                    from_email,
                    to_email,
                    subject,
                    customer_name,
                    customer_phone,
                    listing_title,
                    event_date,
                    message,
                );

                self.update_diagnostics_success();
                Ok(())
            }
        }
    }
}

/// Minimal transactional HTML email for password resets.
fn build_reset_email_html(brand_name: &str, reset_link: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="ar" dir="rtl">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>إعادة تعيين كلمة المرور</title>
</head>
<body style="margin:0;padding:0;background:#f4f4f4;font-family:Arial,sans-serif;">
  <table width="100%" cellpadding="0" cellspacing="0" style="background:#f4f4f4;padding:40px 0;">
    <tr>
      <td align="center">
        <table width="560" cellpadding="0" cellspacing="0" style="background:#fff;border-radius:8px;overflow:hidden;box-shadow:0 2px 8px rgba(0,0,0,0.08);">
          <tr>
            <td style="background:#1a1a2e;padding:32px 40px;text-align:center;">
              <h1 style="color:#fff;margin:0;font-size:22px;">{brand}</h1>
            </td>
          </tr>
          <tr>
            <td style="padding:40px;">
              <h2 style="color:#1a1a2e;font-size:20px;margin:0 0 16px;">طلب إعادة تعيين كلمة المرور</h2>
              <p style="color:#555;line-height:1.6;margin:0 0 24px;">
                لقد تلقينا طلبًا لإعادة تعيين كلمة المرور لحسابك. اضغط على الزر أدناه للمتابعة.
              </p>
              <p style="color:#555;line-height:1.6;margin:0 0 24px;">
                يصلح هذا الرابط لمدة <strong>ساعة واحدة</strong> فقط ولمرة واحدة فقط.
              </p>
              <div style="text-align:center;margin:32px 0;">
                <a href="{link}"
                   style="display:inline-block;background:#d4af37;color:#1a1a2e;font-weight:700;font-size:16px;padding:14px 36px;border-radius:6px;text-decoration:none;">
                  إعادة تعيين كلمة المرور
                </a>
              </div>
              <p style="color:#999;font-size:13px;line-height:1.5;">
                إذا لم تطلب إعادة تعيين كلمة المرور، يمكنك تجاهل هذا البريد بأمان — لن يتغيّر حسابك.
              </p>
            </td>
          </tr>
          <tr>
            <td style="background:#f9f9f9;padding:20px 40px;border-top:1px solid #eee;text-align:center;">
              <p style="color:#bbb;font-size:12px;margin:0;">
                © 2026 {brand}. جميع الحقوق محفوظة.
              </p>
            </td>
          </tr>
        </table>
      </td>
    </tr>
  </table>
</body>
</html>"#,
        brand = brand_name,
        link = reset_link,
    )
}
