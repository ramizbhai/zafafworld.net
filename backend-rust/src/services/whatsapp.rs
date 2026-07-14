use serde_json::json;
use tracing::{error, info, warn};

#[derive(Debug, Clone, serde::Serialize)]
pub struct WhatsappDiagnostics {
    pub configured: bool,
    pub token_valid: Option<bool>,
    pub phone_number_id_valid: Option<bool>,
    pub template_name: String,
    pub last_delivery: Option<chrono::DateTime<chrono::Utc>>,
    pub last_error: Option<String>,
}

pub struct WhatsappService {
    token: Option<String>,
    phone_number_id: Option<String>,
    graph_base_url: String,
    graph_version: String,
    template_name: String,
    language: String,
    client: reqwest::Client,
    diagnostics: std::sync::Arc<std::sync::RwLock<WhatsappDiagnostics>>,
}

impl WhatsappService {
    pub fn from_config(config: &crate::config::AppConfig) -> Self {
        let token = config.whatsapp_token.clone();
        let phone_number_id = config.whatsapp_phone_number_id.clone();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.http_timeout_seconds))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        let app_env = &config.app_environment;
        let is_prod = app_env.to_lowercase() == "production";
        let configured = token.is_some() && phone_number_id.is_some();

        if configured {
            info!(
                "WhatsApp service initialised in PRODUCTION mode (Meta Graph API: {}/{}/{})",
                config.whatsapp_graph_base_url,
                config.whatsapp_graph_version,
                // SAFETY: phone_number_id guaranteed Some when configured is true
                phone_number_id.as_ref().unwrap()
            );
        } else {
            let mut missing = Vec::new();
            if token.is_none() {
                missing.push("WHATSAPP_TOKEN");
            }
            if phone_number_id.is_none() {
                missing.push("WHATSAPP_PHONE_NUMBER_ID");
            }
            if is_prod {
                warn!(
                    "!!! WARNING !!! WhatsApp notification channel is DISABLED in production: missing configuration variables {:?}",
                    missing
                );
            } else {
                info!(
                    "WhatsApp service running in DEVELOPMENT logging-only mode (missing {:?})",
                    missing
                );
            }
        }

        let diagnostics = std::sync::Arc::new(std::sync::RwLock::new(WhatsappDiagnostics {
            configured,
            token_valid: None,
            phone_number_id_valid: None,
            template_name: config.whatsapp_template_new_inquiry.clone(),
            last_delivery: None,
            last_error: None,
        }));

        Self {
            token,
            phone_number_id,
            graph_base_url: config.whatsapp_graph_base_url.clone(),
            graph_version: config.whatsapp_graph_version.clone(),
            template_name: config.whatsapp_template_new_inquiry.clone(),
            language: config.whatsapp_language.clone(),
            client,
            diagnostics,
        }
    }

    pub fn get_diagnostics(&self) -> WhatsappDiagnostics {
        // SAFETY: RwLock read, only fails if poisoned
        self.diagnostics.read().unwrap().clone()
    }

    fn update_diagnostics_success(&self) {
        if let Ok(mut diag) = self.diagnostics.write() {
            diag.token_valid = Some(true);
            diag.phone_number_id_valid = Some(true);
            diag.last_delivery = Some(chrono::Utc::now());
            diag.last_error = None;
        }
    }

    fn update_diagnostics_failure(&self, err: &str, status_code: Option<u16>) {
        if let Ok(mut diag) = self.diagnostics.write() {
            diag.last_error = Some(err.to_string());
            if let Some(sc) = status_code {
                if sc == 401 {
                    diag.token_valid = Some(false);
                } else if sc == 404 || sc == 400 {
                    diag.phone_number_id_valid = Some(false);
                }
            }
        }
    }

    pub async fn send_inquiry_alert(
        &self,
        vendor_phone: &str,
        customer_name: &str,
        customer_phone: &str,
        event_date: &str,
        message: &str,
    ) -> Result<(), String> {
        if vendor_phone.trim().is_empty() {
            return Ok(());
        }

        if let (Some(token), Some(phone_id)) = (&self.token, &self.phone_number_id) {
            let url = format!(
                "{}/{}/{}/messages",
                self.graph_base_url, self.graph_version, phone_id
            );

            let clean_to = vendor_phone.replace(['+', ' ', '-'], "");

            let payload = json!({
                "messaging_product": "whatsapp",
                "to": clean_to,
                "type": "template",
                "template": {
                    "name": self.template_name,
                    "language": { "code": self.language },
                    "components": [
                        {
                            "type": "body",
                            "parameters": [
                                { "type": "text", "text": customer_name },
                                { "type": "text", "text": customer_phone },
                                { "type": "text", "text": event_date },
                                { "type": "text", "text": message }
                            ]
                        }
                    ]
                }
            });

            match self
                .client
                .post(&url)
                .bearer_auth(token)
                .json(&payload)
                .send()
                .await
            {
                Ok(resp) => {
                    let status = resp.status();
                    let sc = status.as_u16();
                    if status.is_success() {
                        info!(
                            "Successfully sent WhatsApp inquiry alert to vendor {}",
                            vendor_phone
                        );
                        self.update_diagnostics_success();
                        Ok(())
                    } else {
                        let err_body = resp.text().await.unwrap_or_default();
                        let err_msg =
                            format!("Meta WhatsApp API returned error {}: {}", status, err_body);
                        error!("{}", err_msg);
                        self.update_diagnostics_failure(&err_msg, Some(sc));
                        Err(err_msg)
                    }
                }
                Err(err) => {
                    let err_msg = format!("Failed to send WhatsApp message via Meta API: {}", err);
                    error!("{}", err_msg);
                    self.update_diagnostics_failure(&err_msg, None);
                    Err(err_msg)
                }
            }
        } else {
            info!(
                "\n==============================================================\n\
                 [DEV WHATSAPP — NOT SENT] New Inquiry Alert\n\
                 --------------------------------------------------------------\n\
                 To (Vendor) : {}\n\
                 Template    : {}\n\
                 Body Params : [{}, {}, {}, {}]\n\
                 ==============================================================",
                vendor_phone,
                self.template_name,
                customer_name,
                customer_phone,
                event_date,
                message
            );
            self.update_diagnostics_success();
            Ok(())
        }
    }
}
