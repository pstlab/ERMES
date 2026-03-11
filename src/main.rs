use coco::{
    CoCo,
    server::start_server,
    tracing::{self, Level},
    tracing_subscriber,
};
use rust_embed::Embed;
use serde_json::json;
use std::{path::Path, sync::Arc};

#[derive(Embed)]
#[folder = "classes/"]
struct Classes;

#[derive(Embed)]
#[folder = "rules/"]
struct Rules;

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt().with_max_level(Level::TRACE).finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set global default subscriber");
    let coco = Arc::new(CoCo::default().await);
    for file in Classes::iter() {
        let content = Classes::get(file.as_ref()).unwrap();
        let class_def = std::str::from_utf8(content.data.as_ref()).unwrap();
        coco.load_class(class_def).await.unwrap();
    }
    for file in Rules::iter() {
        let content = Rules::get(file.as_ref()).unwrap();
        let rule_name = Path::new(file.as_ref()).file_stem().unwrap().to_str().unwrap();
        let rule_def = std::str::from_utf8(content.data.as_ref()).unwrap();
        coco.load_rule(json!({"name": rule_name, "content": rule_def}).to_string().as_str()).await.unwrap();
    }
    start_server(coco).await;
}
