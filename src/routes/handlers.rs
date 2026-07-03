use actix_web::{HttpResponse, Responder, get, post, web};
use serde::Deserialize;
use tera::{Context, Tera};

use crate::linguakit_client::analizar_linguakit_local;
use crate::models::{AnalisisForm, BatchAnalisisRequest};
use crate::stanford_client::analizar_stanford_real;

use super::analysis::ejecutar_analisis;
use super::batch::ejecutar_analisis_lote;

#[derive(Debug, Deserialize)]
pub struct TextoQuery {
    pub texto: String,
}

#[get("/health")]
pub async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "lingua-analytica"
    }))
}

#[get("/")]
pub async fn index(tmpl: web::Data<Tera>) -> impl Responder {
    let ctx = Context::new();

    match tmpl.render("index.html", &ctx) {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),

        Err(err) => HttpResponse::InternalServerError()
            .content_type("text/plain")
            .body(format!("Error renderizando index.html: {err}")),
    }
}

#[get("/metodologia")]
pub async fn metodologia(tmpl: web::Data<Tera>) -> impl Responder {
    let ctx = Context::new();

    match tmpl.render("metodologia.html", &ctx) {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),

        Err(err) => HttpResponse::InternalServerError()
            .content_type("text/plain")
            .body(format!("Error renderizando metodologia.html: {err}")),
    }
}

#[post("/analizar")]
pub async fn analizar(form: web::Form<AnalisisForm>, tmpl: web::Data<Tera>) -> impl Responder {
    let texto = form.texto.trim();

    if texto.is_empty() {
        return HttpResponse::BadRequest()
            .content_type("text/html")
            .body(r#"
                <div style="padding:16px;border:1px solid #991b1b;background:#450a0a;color:#fecaca;">
                    Error: la entrada de texto no puede estar vacía.
                </div>
            "#);
    }

    let resultado = ejecutar_analisis(texto, &form.tipo).await;

    let mut ctx = Context::new();
    ctx.insert("r", &resultado);

    match tmpl.render("partials/resultado.html", &ctx) {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),

        Err(err) => HttpResponse::InternalServerError()
            .content_type("text/plain")
            .body(format!("Error renderizando resultado.html: {err}")),
    }
}

#[post("/api/analizar")]
pub async fn analizar_json(form: web::Form<AnalisisForm>) -> impl Responder {
    let texto = form.texto.trim();

    if texto.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "ok": false,
            "error": "La entrada de texto no puede estar vacía."
        }));
    }

    HttpResponse::Ok().json(ejecutar_analisis(texto, &form.tipo).await)
}

#[post("/api/analizar-lote")]
pub async fn analizar_lote_json(payload: web::Json<BatchAnalisisRequest>) -> impl Responder {
    let texto = payload.texto.trim();
    let tipo = payload.tipo.trim();

    match ejecutar_analisis_lote(texto, tipo).await {
        Ok(resultado) => HttpResponse::Ok().json(resultado),
        Err(error) => HttpResponse::BadRequest().json(serde_json::json!({
            "ok": false,
            "error": error
        })),
    }
}

#[get("/api/linguakit/local")]
pub async fn probar_linguakit_local(query: web::Query<TextoQuery>) -> impl Responder {
    let texto = query.texto.trim();

    if texto.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "ok": false,
            "error": "Debe enviar el parámetro texto"
        }));
    }

    let resultado = analizar_linguakit_local(texto).await;

    HttpResponse::Ok().json(resultado)
}

#[get("/api/stanford/local")]
pub async fn probar_stanford_local(query: web::Query<TextoQuery>) -> impl Responder {
    let texto = query.texto.trim();

    if texto.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "ok": false,
            "error": "Debe enviar el parámetro texto"
        }));
    }

    let resultado = analizar_stanford_real(texto).await;

    HttpResponse::Ok().json(resultado)
}
