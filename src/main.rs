use coco::{
    CoCo, CoCoModule,
    db::mongodb::MongoDB,
    kb::clips::CLIPSKnowledgeBase,
    model::{Class, Rule},
    mqtt::MQTTModule,
    server::{secure::secure_coco_router, secure_db::UsersDB},
};
use rust_embed::Embed;
use std::{collections::HashSet, path::Path};
use tower_http::services::{ServeDir, ServeFile};
use tracing::{Level, error, info, trace};

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

    let classes: HashSet<String> = coco
        .get_classes()
        .await
        .unwrap_or_else(|e| {
            error!("Failed to retrieve classes from CoCo: {}", e);
            std::process::exit(1);
        })
        .into_iter()
        .map(|c| c.name)
        .collect();
    let rules: HashSet<String> = coco
        .get_rules()
        .await
        .unwrap_or_else(|e| {
            error!("Failed to retrieve rules from CoCo: {}", e);
            std::process::exit(1);
        })
        .into_iter()
        .map(|r| r.name)
        .collect();

    info!("Loading classes from embedded resources...");
    for file in Classes::iter().filter(|f| f.as_ref().ends_with(".json")) {
        if let Some(content) = Classes::get(file.as_ref()) {
            if let Ok(class_def) = std::str::from_utf8(content.data.as_ref()) {
                if let Ok(class) = serde_json::from_str::<Class>(class_def) {
                    if classes.contains(&class.name) {
                        trace!("Class {} already exists", class.name);
                    } else {
                        trace!("Class {} does not exist, will attempt to create", class.name);
                        if let Err(e) = coco.create_class(class.clone()).await {
                            error!("Failed to create class {}: {}", file.as_ref(), e);
                        } else {
                            trace!("Class {} created successfully", class.name);
                        }
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

    info!("Loading rules from embedded resources...");
    for file in Rules::iter().filter(|f| f.as_ref().ends_with(".clp")) {
        let rule_name = Path::new(file.as_ref()).file_stem().unwrap().to_str().unwrap();
        if rules.contains(rule_name) {
            trace!("Rule {} already exists", rule_name);
        } else {
            trace!("Rule {} does not exist, will attempt to create", rule_name);
            if let Some(content) = Rules::get(file.as_ref()) {
                if let Ok(rule_def) = std::str::from_utf8(content.data.as_ref()) {
                    if let Err(e) = coco.create_rule(Rule { name: rule_name.to_string(), content: rule_def.to_string() }).await {
                        error!("Failed to create rule {}: {}", file.as_ref(), e);
                    } else {
                        trace!("Rule {} created successfully", rule_name);
                    }
                } else {
                    error!("Failed to parse rule file {}: {}", file.as_ref(), "UTF-8 parsing error");
                }
            } else {
                error!("Failed to load rule file: {}", file.as_ref());
            }
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
