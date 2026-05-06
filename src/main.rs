use coco::{
    CoCo, CoCoModule,
    db::mongodb::MongoDB,
    kb::clips::CLIPSKnowledgeBase,
    model::{Class, Rule},
    mqtt::MQTTModule,
    server::{secure::secure_coco_router, secure_db::UsersDB},
};
use rust_embed::Embed;
use tower_http::services::{ServeDir, ServeFile};
use tracing::{Level, error, info};

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

    let db = MongoDB::default().await.unwrap_or_else(|e| {
        error!("Failed to set up MongoDB: {}", e);
        std::process::exit(1);
    });
    let (kb, event) = CLIPSKnowledgeBase::new();
    let modules: Vec<Box<dyn CoCoModule<MongoDB, CLIPSKnowledgeBase>>> = vec![Box::new(MQTTModule::default())];

    let coco = CoCo::new(db.clone(), kb, event, modules).await;

    for file in Classes::iter() {
        if let Some(content) = Classes::get(file.as_ref()) {
            if let Ok(class_def) = std::str::from_utf8(content.data.as_ref()) {
                if let Ok(class) = serde_json::from_str::<Class>(class_def) {
                    if let Ok(c_class) = coco.get_class(class.name.clone()).await {
                        if c_class.is_none() {
                            info!("Creating class: {}", class.name);
                            if let Err(e) = coco.create_class(class).await {
                                error!("Failed to create class {}: {}", file.as_ref(), e);
                            }
                        } else {
                            info!("Class {} already exists, skipping creation", class.name);
                        }
                    } else {
                        error!("Failed to get class {}: {}", class.name, "Database error");
                    }
                } else {
                    error!("Failed to deserialize class {}: {}", file.as_ref(), "Deserialization error");
                }
            } else {
                error!("Failed to parse class file {}: {}", file.as_ref(), "UTF-8 parsing error");
            }
        } else {
            error!("Failed to load class file: {}", file.as_ref());
        }
    }

    for file in Rules::iter() {
        if let Some(content) = Rules::get(file.as_ref()) {
            if let Ok(rule_def) = std::str::from_utf8(content.data.as_ref()) {
                if let Ok(rule) = serde_json::from_str::<Rule>(rule_def) {
                    if let Ok(c_rule) = coco.get_rule(rule.name.clone()).await {
                        if c_rule.is_none() {
                            info!("Creating rule: {}", rule.name);
                            if let Err(e) = coco.create_rule(rule).await {
                                error!("Failed to create rule {}: {}", file.as_ref(), e);
                            }
                        } else {
                            info!("Rule {} already exists, skipping creation", rule.name);
                        }
                    } else {
                        error!("Failed to get rule {}: {}", rule.name, "Database error");
                    }
                } else {
                    error!("Failed to deserialize rule {}: {}", file.as_ref(), "Deserialization error");
                }
            } else {
                error!("Failed to parse rule file {}: {}", file.as_ref(), "UTF-8 parsing error");
            }
        } else {
            error!("Failed to load rule file: {}", file.as_ref());
        }
    }

    let app = secure_coco_router(
        coco,
        UsersDB::default().await.unwrap_or_else(|e| {
            error!("Failed to set up UsersDB: {}", e);
            std::process::exit(1);
        }),
    )
    .await;
    let app = app.route_service("/favicon.ico", ServeFile::new("gui/favicon.ico")).nest_service("/assets", ServeDir::new("gui/assets")).fallback_service(ServeDir::new("gui").not_found_service(ServeFile::new("gui/index.html")));
    let port = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(3000);
    info!("Starting CoCo server on port {}", port);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
