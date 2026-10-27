use async_trait::async_trait;
use chrono::Utc;
use coco::{
    CoCo, CoCoModule,
    db::Database,
    kb::KnowledgeBase,
    model::{CoCoError, CoCoValue},
};
use google_cloud_pubsub::client::Subscriber;
use serde_json::Value;
use std::{collections::HashMap, env};
use tracing::{error, info, trace, warn};

pub struct SeremyModule {
    subscription_name: String,
}

impl SeremyModule {
    pub fn new(subscription_name: String) -> Self {
        Self { subscription_name }
    }
}

impl Default for SeremyModule {
    fn default() -> Self {
        let subscription_name = env::var("SEREMY_SUBSCRIPTION_NAME").unwrap_or_else(|_| {
            let project = env::var("GCLOUD_PROJECT").unwrap_or_else(|_| "seremy-project-id".to_string());
            let sub = env::var("PUBSUB_SUBSCRIPTION").unwrap_or_else(|_| "seremy-subscription-name".to_string());
            format!("projects/{}/subscriptions/{}", project, sub)
        });
        Self::new(subscription_name)
    }
}

#[async_trait]
impl<DB: Database, KB: KnowledgeBase> CoCoModule<DB, KB> for SeremyModule {
    async fn init(&self, _db: DB, _kb: KB, coco: CoCo) -> Result<(), CoCoError> {
        info!("Initializing SeremyModule with subscription: {}", self.subscription_name);

        let client = Subscriber::builder().build().await.map_err(|e| CoCoError::ConfigurationError(format!("Failed to create Pub/Sub subscriber: {}", e)))?;

        let subscription_name = self.subscription_name.clone();

        tokio::spawn(async move {
            let mut stream = client.subscribe(subscription_name).build();

            while let Some(msg) = stream.next().await {
                match msg {
                    Ok((msg, hndl)) => {
                        trace!("Received message: {:?}", msg);
                        if let Ok(data) = serde_json::from_slice::<Value>(&msg.data) {
                            let Some(seremy_id) = data.get("deviceSerialNumber").and_then(|v| v.as_str()) else {
                                error!("Message missing deviceSerialNumber: {:?}", data);
                                hndl.ack();
                                continue;
                            };
                            let Some(coco_id) = coco.get_objects().await.unwrap_or_default().into_iter().find(|o| matches!(o.properties.as_ref().and_then(|p| p.get("seremy_id")), Some(CoCoValue::String(v) | CoCoValue::Symbol(v)) if v == seremy_id)).map(|o| o.id.expect("Object ID should be present")) else {
                                warn!("No CoCo object found for Seremy device {}, skipping message", seremy_id);
                                hndl.ack();
                                continue;
                            };

                            let mut values: HashMap<String, CoCoValue> = HashMap::new();

                            match data.get("msgType").and_then(|v| v.as_str()) {
                                Some("ALARM") => {
                                    if let Some(alarm_type) = data.get("alarm_type").and_then(|v| v.as_str()) {
                                        values.insert("alarm".to_string(), CoCoValue::Symbol(alarm_type.to_string()));
                                    }
                                    if let Some(source) = data.get("alarm_source_type").and_then(|v| v.as_str()) {
                                        values.insert("alarm_source".to_string(), CoCoValue::Symbol(source.to_string()));
                                    }
                                }
                                Some("HRM") => {
                                    if let Some(avg_bpm) = data.get("avgBpm").and_then(|v| v.as_i64()) {
                                        values.insert("avg_bpm".to_string(), CoCoValue::Int(avg_bpm));
                                    }
                                    if let Some(max_bpm) = data.get("maxBpm").and_then(|v| v.as_i64()) {
                                        values.insert("max_bpm".to_string(), CoCoValue::Int(max_bpm));
                                    }
                                    if let Some(min_bpm) = data.get("minBpm").and_then(|v| v.as_i64()) {
                                        values.insert("min_bpm".to_string(), CoCoValue::Int(min_bpm));
                                    }
                                }
                                Some("STEPS") => {
                                    if let Some(steps) = data.get("total_steps").and_then(|v| v.as_i64()) {
                                        values.insert("steps".to_string(), CoCoValue::Int(steps));
                                    }
                                }
                                _ => {}
                            }

                            let pos_obj = data.get("position").unwrap_or(&data);
                            if let (Some(lat), Some(lon)) = (pos_obj.get("latitude"), pos_obj.get("longitude")) {
                                let lat_val = lat.as_str().and_then(|s| s.parse::<f64>().ok()).or_else(|| lat.as_f64()).unwrap_or(0.0);
                                let lon_val = lon.as_str().and_then(|s| s.parse::<f64>().ok()).or_else(|| lon.as_f64()).unwrap_or(0.0);

                                values.insert("latitude".to_string(), CoCoValue::Float(lat_val));
                                values.insert("longitude".to_string(), CoCoValue::Float(lon_val));

                                if let Some(fix) = pos_obj.get("fixType").and_then(|v| v.as_str()) {
                                    values.insert("fix_type".to_string(), CoCoValue::String(fix.to_string()));
                                }
                            }

                            if !values.is_empty() {
                                trace!("Adding values for device {}: {:?}", coco_id, values);
                                if let Err(e) = coco.add_values(coco_id.clone(), values, Utc::now()).await {
                                    error!("Failed to add values for device {}: {}", coco_id, e);
                                }
                            }
                        } else {
                            error!("Failed to parse message data as JSON: {:?}", msg.data);
                        }

                        hndl.ack();
                    }
                    Err(e) => {
                        error!("Error receiving message: {}", e);
                    }
                }
            }
        });

        Ok(())
    }
}
