use std::sync::{Arc, RwLock};

use anyhow::anyhow;
use ava_toolkit::device_message::RadiatorMode;
use log::info;
use reqwest::header;
use serde::Deserialize;

#[derive(Clone, Debug)]
pub struct HeatzyClient {
    application_id: String,
    username: String,
    password: String,
    token: Arc<RwLock<String>>,
}

impl HeatzyClient {
    pub fn new(
        application_id: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
        token: Arc<RwLock<String>>,
    ) -> Self {
        Self {
            application_id: application_id.into(),
            username: username.into(),
            password: password.into(),
            token,
        }
    }

    pub fn current_token(&self) -> anyhow::Result<String> {
        let guard = self
            .token
            .read()
            .map_err(|_| anyhow!("Cannot read current Heatzy token"))?;
        Ok(guard.clone())
    }

    pub async fn set_mode(&self, did: &str, mode: RadiatorMode) -> anyhow::Result<()> {
        let current_token = self.current_token()?;

        info!("Using existing Heatzy token for control API call");
        match self
            .set_mode_with_token(&current_token, did, mode)
            .await
        {
            Ok(()) => Ok(()),
            Err(HeatzyCallError::HttpStatus { status, body }) if status.is_client_error() => {
                info!("🔄 Reconnecting to Heatzy to refresh token");
                info!(
                    "Heatzy token rejected with status {} and body [{}], trying login before retrying command",
                    status, body
                );
                let refreshed_token = self.login().await?;

                {
                    let mut guard = self
                        .token
                        .write()
                        .map_err(|_| anyhow!("Cannot update Heatzy token"))?;
                    *guard = refreshed_token.clone();
                }
                info!("Heatzy token refreshed in memory");

                self.set_mode_with_token(&refreshed_token, did, mode)
                    .await
                    .map_err(|e| anyhow!("Heatzy error after relogin: {}", e))
            }
            Err(e) => Err(anyhow!("Heatzy error: {}", e)),
        }
    }

    async fn set_mode_with_token(
        &self,
        heatzy_token: &str,
        did: &str,
        mode: RadiatorMode,
    ) -> Result<(), HeatzyCallError> {
        let h_mode = match mode {
            RadiatorMode::CFT => 0,
            RadiatorMode::ECO => 1,
            RadiatorMode::FRO => 2,
            RadiatorMode::STOP => 3,
        };

        let data = serde_json::json!({
            "attrs": {
                "mode": h_mode
            }
        });

        let url = format!("https://euapi.gizwits.com/app/control/{}", did);

        let mut custom_header = header::HeaderMap::new();
        custom_header.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("reqwest"),
        );
        custom_header.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        custom_header.insert(
            "X-Gizwits-Application-Id",
            self.application_id.parse().unwrap(),
        );
        custom_header.insert("X-Gizwits-User-token", heatzy_token.parse().unwrap());

        let client = reqwest::Client::new();
        let response = client
            .post(url)
            .headers(custom_header)
            .json(&data)
            .send()
            .await
            .map_err(HeatzyCallError::Request)?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "<unreadable response body>".to_string());
            Err(HeatzyCallError::HttpStatus {
                status,
                body: body.trim().to_string(),
            })
        }
    }

    async fn login(&self) -> anyhow::Result<String> {
        let url = "https://euapi.gizwits.com/app/login";
        let body = serde_json::json!({
            "username": self.username,
            "password": self.password,
        });
        let redacted_body = serde_json::json!({
            "username": self.username,
            "password": "***",
        });

        let mut custom_header = header::HeaderMap::new();
        custom_header.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("reqwest"),
        );
        custom_header.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        custom_header.insert(
            "X-Gizwits-Application-Id",
            self.application_id.parse().unwrap(),
        );

        info!("🔐 Heatzy relogin request");
        info!("Heatzy relogin URL: {}", url);
        info!(
            "Heatzy relogin headers: User-Agent=reqwest, Content-Type=application/json, X-Gizwits-Application-Id={}",
            self.application_id
        );
        info!("Heatzy relogin body: {}", redacted_body);

        #[derive(Debug, Deserialize)]
        struct LoginResponse {
            token: String,
        }

        let client = reqwest::Client::new();
        let response = client
            .post(url)
            .headers(custom_header)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let response_body = response
                .text()
                .await
                .unwrap_or_else(|_| "<unreadable response body>".to_string());
            return Err(anyhow!(
                "Heatzy login failed with status {} body {}",
                status,
                response_body.trim()
            ));
        }

        let login_response: LoginResponse = response.json().await?;
        Ok(login_response.token)
    }
}

enum HeatzyCallError {
    Request(reqwest::Error),
    HttpStatus {
        status: reqwest::StatusCode,
        body: String,
    },
}

impl std::fmt::Display for HeatzyCallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HeatzyCallError::Request(e) => write!(f, "request error: {}", e),
            HeatzyCallError::HttpStatus { status, body } => {
                if body.is_empty() {
                    write!(f, "status {}", status)
                } else {
                    write!(f, "status {} body {}", status, body)
                }
            }
        }
    }
}
