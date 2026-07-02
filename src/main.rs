mod decision_engine;
mod linguakit_client;
mod models;
mod routes;
mod stanford_client;

use actix_files::Files;
use actix_web::{App, HttpServer, web};
use dotenvy::dotenv;
use std::env;
use tera::Tera;

use routes::{
    analizar, analizar_json, index, metodologia, probar_linguakit_local, probar_stanford_local,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let host = env::var("APP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("APP_PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_addr = format!("{host}:{port}");

    let mut tera = Tera::new();

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
            .service(index)
            .service(metodologia)
            .service(analizar)
            .service(analizar_json)
            .service(probar_linguakit_local)
            .service(probar_stanford_local)
            .service(Files::new("/static", "./static").show_files_listing())
    })
    .bind(bind_addr)?
    .run()
    .await
}
