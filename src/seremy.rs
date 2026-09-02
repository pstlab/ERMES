use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use coco::{
    CoCo, CoCoModule,
    db::Database,
    kb::KnowledgeBase,
    model::{CoCoError, CoCoValue},
};
use google_cloud_pubsub::client::Subscriber;
use serde_json::Value;
use std::{collections::HashMap, env};
use tracing::{error, info, trace};

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
        let subscription_name = env::var("SEREMY_SUBSCRIPTION_NAME").expect("SEREMY_SUBSCRIPTION_NAME environment variable not set");
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
                            let Some(device_id) = data.get("deviceSerialNumber").and_then(|v| v.as_str()) else {
                                error!("Message missing deviceSerialNumber: {:?}", data);
                                hndl.ack();
                                continue;
                            };

                            let Some(timestamp) = data.get("msgTimeStamp").and_then(|value| value.as_i64()).and_then(|seconds| Utc.timestamp_opt(seconds, 0).single()) else {
                                error!("Message missing or invalid msgTimeStamp: {:?}", data);
                                hndl.ack();
                                continue;
                            };

                            let mut values: HashMap<String, CoCoValue> = HashMap::new();

                            match data.get("msgType").and_then(|v| v.as_str()) {
                                Some("ALARM") => {
                                    if let Some(alarm_type) = data.get("alarm_type").and_then(|v| v.as_str()) {
                                        values.insert("alarm".to_string(), CoCoValue::String(alarm_type.to_string()));
                                    }
                                    if let Some(source) = data.get("alarm_source_type").and_then(|v| v.as_str()) {
                                        values.insert("alarm_source".to_string(), CoCoValue::String(source.to_string()));
                                    }
                                }
                                _ => {}
                            }

                            if let Some(position) = data.get("position") {
                                if let (Some(lat), Some(lon)) = (position.get("latitude"), position.get("longitude")) {
                                    let lat_val = lat.as_str().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                                    let lon_val = lon.as_str().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);

                                    values.insert("latitude".to_string(), CoCoValue::Float(lat_val));
                                    values.insert("longitude".to_string(), CoCoValue::Float(lon_val));

                                    if let Some(fix) = position.get("fixType").and_then(|v| v.as_str()) {
                                        values.insert("fix_type".to_string(), CoCoValue::String(fix.to_string()));
                                    }
                                }
                            }

                            if !values.is_empty() {
                                trace!("Adding values for device {}: {:?} at timestamp {}", device_id, values, timestamp);
                                if let Err(e) = coco.add_values(device_id.to_string(), values, timestamp).await {
                                    error!("Failed to add values for device {}: {}", device_id, e);
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
