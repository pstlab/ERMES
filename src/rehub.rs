use async_trait::async_trait;
use chrono::Utc;
use coco::{CoCo, CoCoModule, db::Database, kb::KnowledgeBase, model::CoCoError};
use rand::Rng;
use reqwest::Client;
use rsa::{Oaep, RsaPublicKey, pkcs8::DecodePublicKey, rand_core::OsRng, sha2::Sha256};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::{self, MissedTickBehavior};
use tracing::{error, info, trace};

const ACTION: &str = "/pattient/exercises/bydate";

#[derive(Serialize)]
struct InnerData<'a> {
    date: &'a str,
}

#[derive(Serialize)]
struct PlainPayload<'a> {
    data: InnerData<'a>,
    timestamp: u64,
    nonce: String,
    action: &'a str,
}

#[derive(Serialize)]
struct EncryptedRequest {
    data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReHubResponse {
    pub now: String,
    pub timezone: String,
    pub patients: Vec<PatientExercises>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatientExercises {
    pub patient_id: String,
    pub exercises: Vec<Exercise>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Exercise {
    pub exercise_id: String,
    pub protocol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_duration: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct ApiErrorResponse {
    error: Option<String>,
    message: Option<String>,
}

#[derive(Clone)]
pub struct ReHubModule {
    http_client: Client,
    endpoint_url: String,
    client_id: String,
    public_key: RsaPublicKey,
}

impl Default for ReHubModule {
    fn default() -> Self {
        let endpoint_url = std::env::var("REHUB_ENDPOINT_URL").unwrap_or_else(|_| "https://rehub.inrca.it/api".to_string());
        let client_id = std::env::var("REHUB_CLIENT_ID").unwrap_or_else(|_| "default_client_id".to_string());
        let pem_path = std::env::var("REHUB_PUBLIC_KEY_CREDENTIALS").unwrap_or_else(|_| "config/rehub_public_key.pem".to_string());
        let public_key = RsaPublicKey::read_public_key_pem_file(pem_path)
            .map_err(|e| {
                error!("Failed to read public key PEM file: {}", e);
                std::process::exit(1);
            })
            .unwrap_or_else(|e| {
                error!("Failed to read public key PEM file: {}", e);
                std::process::exit(1);
            });

        Self::new(endpoint_url, client_id, public_key).unwrap_or_else(|e| {
            error!("Failed to initialize ReHubModule: {}", e);
            std::process::exit(1);
        })
    }
}

impl ReHubModule {
    pub fn new(endpoint_url: impl Into<String>, client_id: impl Into<String>, public_key: RsaPublicKey) -> Result<Self, CoCoError> {
        let http_client = Client::builder().timeout(Duration::from_secs(60)).build().map_err(|e| CoCoError::ConfigurationError(format!("Failed to build HTTP client: {e}")))?;
        Ok(Self { http_client, endpoint_url: endpoint_url.into(), client_id: client_id.into(), public_key })
    }

    fn build_encrypted_request(&self, date: &str) -> Result<EncryptedRequest, CoCoError> {
        let mut nonce_bytes = [0u8; 16];
        rand::rng().fill_bytes(&mut nonce_bytes);
        let nonce = hex::encode(nonce_bytes);

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| CoCoError::ConfigurationError(format!("System clock error: {e}")))?.as_secs();

        let payload = PlainPayload { data: InnerData { date }, timestamp, nonce, action: ACTION };

        let plaintext = serde_json::to_vec(&payload).map_err(|e| CoCoError::JsonParseError(format!("Payload serialization error: {e}")))?;

        let padding = Oaep::new::<Sha256>();
        let ciphertext = self.public_key.encrypt(&mut OsRng, padding, &plaintext).map_err(|e| CoCoError::ConfigurationError(format!("RSA-OAEP encryption error: {e}")))?;

        Ok(EncryptedRequest { data: hex::encode(ciphertext) })
    }

    pub async fn fetch_exercises_by_date(&self, date: &str) -> Result<ReHubResponse, CoCoError> {
        let body = self.build_encrypted_request(date)?;

        let response = self.http_client.post(&self.endpoint_url).header("client-id", &self.client_id).json(&body).send().await.map_err(|e| CoCoError::ConfigurationError(format!("ReHub HTTP request failed: {e}")))?;

        let status = response.status();
        if status.is_success() {
            response.json::<ReHubResponse>().await.map_err(|e| CoCoError::JsonParseError(format!("Failed to deserialize ReHub response: {e}")))
        } else {
            let raw_err = response.text().await.unwrap_or_default();
            let err_msg = serde_json::from_str::<ApiErrorResponse>(&raw_err).ok().and_then(|e| e.error.or(e.message)).unwrap_or(raw_err);

            Err(CoCoError::ConfigurationError(format!("ReHub API error (HTTP {status}): {err_msg}")))
        }
    }
}

#[async_trait]
impl<DB: Database, KB: KnowledgeBase> CoCoModule<DB, KB> for ReHubModule {
    async fn init(&self, _db: DB, _kb: KB, _coco: CoCo) -> Result<(), CoCoError> {
        info!("Initializing ReHubModule with endpoint: {} and client_id: {}", self.endpoint_url, self.client_id);

        let module = self.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(60));
            interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

            loop {
                interval.tick().await;

                let today_utc = Utc::now().format("%Y-%m-%d").to_string();
                trace!("Polling ReHub for exercises on date: {}", today_utc);

                match module.fetch_exercises_by_date(&today_utc).await {
                    Ok(response) => {
                        trace!("Received ReHub response: {:?}", response);
                    }
                    Err(e) => {
                        error!("Error fetching exercises from ReHub: {}", e);
                    }
                }
            }
        });

        Ok(())
    }
}
