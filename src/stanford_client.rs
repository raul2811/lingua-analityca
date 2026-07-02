use crate::models::{Dependencia, StanfordResult};
use reqwest::Client;
use serde_json::Value;
use std::env;
use urlencoding::encode;

pub async fn analizar_stanford_real(texto: &str) -> StanfordResult {
    let base_url = env::var("STANFORD_URL")
        .unwrap_or_else(|_| "http://localhost:9000".to_string());

    let properties = serde_json::json!({
        "annotators": "tokenize,ssplit,mwt,pos,lemma,depparse",
        "outputFormat": "json"
    });

    let url = format!("{}/?properties={}", base_url, encode(&properties.to_string()));

    let client = Client::new();

    let response = client
        .post(&url)
        .body(texto.to_string())
        .send()
        .await;

    let Ok(resp) = response else {
        return StanfordResult {
            estado: "ERROR STANFORD HTTP".to_string(),
            tokens_pos: vec![],
            dependencias: vec![],
            raw_json: serde_json::json!({
                "ok": false,
                "error": "No se pudo conectar con Stanford CoreNLP en localhost:9000."
            }),
        };
    };

    let status = resp.status().as_u16();
    let body = resp.text().await.unwrap_or_default();

    if status != 200 {
        return StanfordResult {
            estado: format!("ERROR STANFORD {}", status),
            tokens_pos: vec![],
            dependencias: vec![],
            raw_json: serde_json::json!({
                "ok": false,
                "status": status,
                "body": body
            }),
        };
    }

    let json: Value = match serde_json::from_str(&body) {
        Ok(json) => json,
        Err(_) => {
            return StanfordResult {
                estado: "ERROR STANFORD JSON".to_string(),
                tokens_pos: vec![],
                dependencias: vec![],
                raw_json: serde_json::json!({
                    "ok": false,
                    "error": "Stanford respondió, pero no devolvió JSON válido.",
                    "body": body
                }),
            };
        }
    };

    StanfordResult {
        estado: "OK REAL".to_string(),
        tokens_pos: extraer_tokens_stanford(&json),
        dependencias: extraer_dependencias_stanford(&json),
        raw_json: json,
    }
}

fn extraer_tokens_stanford(json: &Value) -> Vec<String> {
    let mut tokens_pos = Vec::new();

    let Some(sentences) = json.get("sentences").and_then(|v| v.as_array()) else {
        return tokens_pos;
    };

    for sentence in sentences {
        let Some(tokens) = sentence.get("tokens").and_then(|v| v.as_array()) else {
            continue;
        };

        for token in tokens {
            let word = token.get("word").and_then(|v| v.as_str()).unwrap_or("");
            let lemma = token.get("lemma").and_then(|v| v.as_str()).unwrap_or("");
            let pos = token.get("pos").and_then(|v| v.as_str()).unwrap_or("");

            if !word.is_empty() {
                tokens_pos.push(format!("{}_{}_{}", word, lemma, pos));
            }
        }
    }

    tokens_pos
}

fn extraer_dependencias_stanford(json: &Value) -> Vec<Dependencia> {
    let mut dependencias = Vec::new();

    let Some(sentences) = json.get("sentences").and_then(|v| v.as_array()) else {
        return dependencias;
    };

    for sentence in sentences {
        let Some(deps) = sentence
            .get("basicDependencies")
            .and_then(|v| v.as_array()) else {
            continue;
        };

        for dep in deps {
            let relacion = dep.get("dep").and_then(|v| v.as_str()).unwrap_or("");

            if relacion == "ROOT" || relacion.is_empty() {
                continue;
            }

            let gobernador = dep
                .get("governorGloss")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let dependiente = dep
                .get("dependentGloss")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            dependencias.push(Dependencia {
                relacion: relacion.to_string(),
                gobernador: gobernador.to_string(),
                dependiente: dependiente.to_string(),
            });
        }
    }

    dependencias
}