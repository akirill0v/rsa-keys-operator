#![allow(unused_imports, unused_variables)]
pub use key_generator::*;
use log::{debug, error, info, trace, warn};
use prometheus::{Encoder, TextEncoder};
use std::env;

use crate::settings::Settings;
use actix_web::{get, App, HttpServer, Responder};
use actix_web::{
    middleware,
    web::{self, Data},
    HttpRequest, HttpResponse,
};

#[get("/metrics")]
async fn metrics(c: Data<Controller>, _req: HttpRequest) -> impl Responder {
    let metrics = c.metrics();
    let encoder = TextEncoder::new();
    let mut buffer = vec![];
    encoder.encode(&metrics, &mut buffer).unwrap();
    HttpResponse::Ok().body(buffer)
}

#[get("/health")]
async fn health(_: HttpRequest) -> impl Responder {
    HttpResponse::Ok().json("healthy")
}

#[get("/")]
async fn index(c: Data<Controller>, _req: HttpRequest) -> impl Responder {
    let state = c.state().unwrap();
    HttpResponse::Ok().json(state)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    // Logging
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "actix_web=info,key_generator=info,kube=debug");
    }
    env_logger::init();

    let config_path =
        env::var("CONTROLLER_CONFIG").unwrap_or_else(|_| "config/default.yaml".into());
    info!("Try to read config from {}", config_path);
    let settings = Settings::new(&config_path).expect("Failed to load controller config");

    let cfg = if let Ok(c) = kube::config::incluster_config() {
        c
    } else {
        kube::config::load_kube_config()
            .await
            .expect("Failed to load kube config")
    };
    let c = state::init(cfg, settings)
        .await
        .expect("Failed to initialize controller");

    HttpServer::new(move || {
        App::new()
            .data(c.clone())
            .wrap(middleware::Logger::default().exclude("/health"))
            .service(index)
            .service(health)
            .service(metrics)
    })
    .bind("0.0.0.0:8080")
    .expect("Can not bind to 0.0.0.0:8080")
    .shutdown_timeout(0)
    .run()
    .await
}
