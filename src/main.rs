mod decision_engine;
mod linguakit_client;
mod models;
mod routes;
mod stanford_client;

use actix_files::Files;
use actix_web::{App, HttpServer, web};
use dotenvy::dotenv;
use std::env;
use std::time::Duration;
use tera::{Kwargs, State, Tera, TeraResult, Value};

use routes::{
    analizar, analizar_json, analizar_lote_json, health, index, metodologia,
    probar_linguakit_local, probar_stanford_local,
};

// =======================================================
// Filtro personalizado para serializar valores Tera a JSON
// Compatible con tera = "2.0.0"
// =======================================================
fn json_encode_filter(val: Value, _: Kwargs, _: &State) -> TeraResult<Value> {
    let json_str = serde_json::to_string(&val)
        .map_err(|e| tera::Error::message(format!("Error serializando JSON: {e}")))?;

    Ok(Value::from_serializable(&json_str))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let host = env::var("APP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("APP_PORT").unwrap_or_else(|_| "8080".to_string());

    if env::args().any(|arg| arg == "--healthcheck") {
        return ejecutar_healthcheck(&port).await;
    }

    let bind_addr = format!("{host}:{port}");

    let mut tera = Tera::new();

    // Registramos ambos nombres porque tus plantillas han usado | json y | json_encode
    tera.register_filter("json", json_encode_filter);
    tera.register_filter("json_encode", json_encode_filter);

    tera.add_template_files(vec![
        ("templates/index.html", Some("index.html")),
        ("templates/metodologia.html", Some("metodologia.html")),
        (
            "templates/partials/resultado.html",
            Some("partials/resultado.html"),
        ),
    ])
    .expect("No se pudieron cargar las plantillas Tera");

    println!("Servidor iniciado en http://{bind_addr}");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera.clone()))
            .app_data(web::JsonConfig::default().limit(100 * 1024 * 1024))
            .service(health)
            .service(index)
            .service(metodologia)
            .service(analizar)
            .service(analizar_json)
            .service(analizar_lote_json)
            .service(probar_linguakit_local)
            .service(probar_stanford_local)
            .service(Files::new("/static", "./static").show_files_listing())
    })
    .bind(bind_addr)?
    .run()
    .await
}

async fn ejecutar_healthcheck(port: &str) -> std::io::Result<()> {
    let url = format!("http://127.0.0.1:{port}/health");
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .map_err(std::io::Error::other)?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(std::io::Error::other)?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "healthcheck returned {}",
            response.status()
        )))
    }
}
