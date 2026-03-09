use coco::{
    CoCo, CoCoState,
    server::{
        axum, build_coco_router,
        tower_http::services::{ServeDir, ServeFile},
    },
    tracing::{self, Level, info},
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

#[derive(Clone)]
struct Ermes {
    coco: Arc<CoCo>,
}

impl CoCoState for Ermes {
    fn coco(&self) -> Arc<CoCo> {
        self.coco.clone()
    }
}

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
    let state = Ermes { coco };

    let app = build_coco_router::<Ermes>();
    let app = app.with_state(state).nest_service("/assets", ServeDir::new("gui/dist/assets")).fallback_service(ServeDir::new("gui/dist").not_found_service(ServeFile::new("gui/dist/index.html")));

    let port = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(3000);

    info!("Starting CoCo server on port {}", port);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
