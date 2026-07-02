use actix_web::{get, post, web, HttpResponse, Responder};
use serde::Deserialize;
use tera::{Context, Tera};

use crate::decision_engine::decidir;
use crate::linguakit_client::analizar_linguakit_local;
use crate::models::{AnalisisForm, AnalisisResultado};
use crate::stanford_client::analizar_stanford_real;

#[derive(Debug, Deserialize)]
pub struct TextoQuery {
    pub texto: String,
}

#[get("/")]
pub async fn index(tmpl: web::Data<Tera>) -> impl Responder {
    let ctx = Context::new();

    match tmpl.render("index.html", &ctx) {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html")
            .body(html),

        Err(err) => HttpResponse::InternalServerError()
            .content_type("text/plain")
            .body(format!("Error renderizando index.html: {err}")),
    }
}

#[post("/analizar")]
pub async fn analizar(
    form: web::Form<AnalisisForm>,
    tmpl: web::Data<Tera>,
) -> impl Responder {
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

    // Herramientas reales: Stanford CoreNLP local + Linguakit CLI local
    let stanford = analizar_stanford_real(texto).await;
    let linguakit = analizar_linguakit_local(texto).await;
    let decision = decidir(&form.tipo, &stanford, &linguakit);

    let resultado = AnalisisResultado {
        entrada: texto.to_string(),
        tipo: form.tipo.clone(),
        stanford,
        linguakit,
        decision,
    };

    let mut ctx = Context::new();
    ctx.insert("r", &resultado);

    match tmpl.render("partials/resultado.html", &ctx) {
        Ok(html) => HttpResponse::Ok()
            .content_type("text/html")
            .body(html),

        Err(err) => HttpResponse::InternalServerError()
            .content_type("text/plain")
            .body(format!("Error renderizando resultado.html: {err}")),
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
