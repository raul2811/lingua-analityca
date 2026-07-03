use crate::models::{Dependencia, LinguakitResult, TokenInfo};
use reqwest::Client;
use serde_json::Value;
use std::env;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::fs;
use tokio::process::Command;

const LINGUAKIT_HTTP_TIMEOUT_SECS: u64 = 20;

pub async fn analizar_linguakit_local(texto: &str) -> LinguakitResult {
    if let Some(result) = analizar_linguakit_http(texto).await {
        return result;
    }

    analizar_linguakit_cli(texto).await
}

async fn analizar_linguakit_http(texto: &str) -> Option<LinguakitResult> {
    let base_url =
        env::var("LINGUAKIT_URL").unwrap_or_else(|_| "http://localhost:3002".to_string());

    let mode = env::var("LINGUAKIT_MODE").unwrap_or_else(|_| "dep".to_string());

    let output =
        normalizar_output_api(&env::var("LINGUAKIT_OUTPUT").unwrap_or_else(|_| "-a".to_string()));

    let url = format!("{}/v2.0/{}", base_url.trim_end_matches('/'), mode);
    let client = Client::builder()
        .timeout(Duration::from_secs(LINGUAKIT_HTTP_TIMEOUT_SECS))
        .build()
        .ok()?;
    let mut body = serde_json::json!({
        "text": texto
    });

    if !output.is_empty() {
        body["output"] = Value::String(output);
    }

    let response = client.post(&url).json(&body).send().await.ok()?;
    let status = response.status().as_u16();
    let body = response.text().await.unwrap_or_default();

    if status != 200 {
        return Some(LinguakitResult {
            estado: format!("ERROR LINGUAKIT API {status}"),
            tokens: vec![],
            dependencias: vec![],
            raw_json: serde_json::json!({
                "modo": "http_api",
                "url": url,
                "status": status,
                "body": body
            }),
        });
    }

    let json: Value = match serde_json::from_str(&body) {
        Ok(json) => json,
        Err(_) => Value::String(body.clone()),
    };

    let stdout = linguakit_json_a_texto(&json);

    Some(LinguakitResult {
        estado: "OK API".to_string(),
        tokens: extraer_tokens_linguakit(&stdout),
        dependencias: extraer_dependencias_linguakit(&stdout),
        raw_json: serde_json::json!({
            "modo": "http_api",
            "url": url,
            "response": json,
            "stdout_normalizado": stdout
        }),
    })
}

async fn analizar_linguakit_cli(texto: &str) -> LinguakitResult {
    let linguakit_bin =
        env::var("LINGUAKIT_BIN").unwrap_or_else(|_| "./tools/Linguakit/linguakit".to_string());

    let lang = env::var("LINGUAKIT_LANG").unwrap_or_else(|_| "es".to_string());

    let mode = env::var("LINGUAKIT_MODE").unwrap_or_else(|_| "dep".to_string());

    let output_flag = env::var("LINGUAKIT_OUTPUT").unwrap_or_else(|_| "-a".to_string());

    let input_path = crear_ruta_temporal();

    if let Err(err) = fs::write(&input_path, texto).await {
        return LinguakitResult {
            estado: "ERROR FILE".to_string(),
            tokens: vec![],
            dependencias: vec![],
            raw_json: serde_json::json!({
                "error": format!("No se pudo escribir el archivo temporal: {}", err)
            }),
        };
    }

    let result = Command::new(&linguakit_bin)
        .arg(&mode)
        .arg(&lang)
        .arg(&input_path)
        .arg(&output_flag)
        .output()
        .await;

    let _ = fs::remove_file(&input_path).await;

    let Ok(output_cmd) = result else {
        return LinguakitResult {
            estado: "ERROR EXEC".to_string(),
            tokens: vec![],
            dependencias: vec![],
            raw_json: serde_json::json!({
                "error": "No se pudo ejecutar Linguakit localmente",
                "binario": linguakit_bin
            }),
        };
    };

    let stdout = String::from_utf8_lossy(&output_cmd.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output_cmd.stderr).to_string();

    if !output_cmd.status.success() {
        return LinguakitResult {
            estado: "ERROR LINGUAKIT".to_string(),
            tokens: vec![],
            dependencias: vec![],
            raw_json: serde_json::json!({
                "stdout": stdout,
                "stderr": stderr,
                "status": output_cmd.status.code()
            }),
        };
    }

    let tokens = extraer_tokens_linguakit(&stdout);
    let dependencias = extraer_dependencias_linguakit(&stdout);

    LinguakitResult {
        estado: "OK LOCAL".to_string(),
        tokens,
        dependencias,
        raw_json: serde_json::json!({
            "modo": "local_cli",
            "comando": format!(
                "{} {} {} {:?} {}",
                linguakit_bin,
                mode,
                lang,
                input_path,
                output_flag
            ),
            "stdout": stdout,
            "stderr": stderr
        }),
    }
}

fn normalizar_output_api(output_flag: &str) -> String {
    match output_flag {
        "-a" | "a" | "" => String::new(),
        "-c" | "c" => "c".to_string(),
        "-fa" | "fa" => "fa".to_string(),
        "conll" => "conll".to_string(),
        other => other.trim_start_matches('-').to_string(),
    }
}

fn linguakit_json_a_texto(json: &Value) -> String {
    match json {
        Value::Array(items) => items
            .iter()
            .map(|item| match item {
                Value::String(line) => line.clone(),
                other => other.to_string(),
            })
            .collect::<Vec<_>>()
            .join("\n"),
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

fn crear_ruta_temporal() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    env::temp_dir().join(format!(
        "linguakit_input_{}_{}.txt",
        std::process::id(),
        nanos
    ))
}

fn extraer_tokens_linguakit(stdout: &str) -> Vec<TokenInfo> {
    let mut tokens = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();

        if !line.starts_with("SENT::") {
            continue;
        }

        let contenido = line.trim_start_matches("SENT::");

        for fragmento in contenido.split_whitespace() {
            if !fragmento.contains("_<") {
                continue;
            }

            if let Some(token) = parsear_token_linguakit(fragmento) {
                tokens.push(token);
            }
        }
    }

    tokens
}

fn parsear_token_linguakit(fragmento: &str) -> Option<TokenInfo> {
    let inicio_features = fragmento.find('<')?;
    let fin_features = fragmento.rfind('>')?;

    let cabecera = fragmento[..inicio_features].trim_end_matches('_');
    let features = &fragmento[inicio_features + 1..fin_features];

    let mut partes = cabecera.rsplitn(3, '_');

    let _indice = partes.next().unwrap_or("");
    let categoria = partes.next().unwrap_or("N/D").to_string();
    let token_cabecera = partes.next().unwrap_or("").to_string();

    let token = extraer_atributo(features, "token").unwrap_or(token_cabecera);

    let lema = extraer_atributo(features, "lemma").unwrap_or_else(|| token.to_lowercase());

    Some(TokenInfo {
        token,
        lema,
        categoria,
    })
}

fn extraer_dependencias_linguakit(stdout: &str) -> Vec<Dependencia> {
    let mut dependencias = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();

        if !(line.starts_with('(') && line.ends_with(')')) {
            continue;
        }

        let limpio = line.trim_start_matches('(').trim_end_matches(')');

        let partes: Vec<&str> = limpio.split(';').collect();

        if partes.len() != 3 {
            continue;
        }

        dependencias.push(Dependencia {
            relacion: partes[0].to_string(),
            gobernador: normalizar_nodo_dependencia(partes[1]),
            dependiente: normalizar_nodo_dependencia(partes[2]),
        });
    }

    dependencias
}

fn normalizar_nodo_dependencia(valor: &str) -> String {
    let mut partes = valor.rsplitn(3, '_');

    let _indice = partes.next().unwrap_or("");
    let categoria = partes.next().unwrap_or("");
    let lema = partes.next().unwrap_or(valor);

    if categoria.is_empty() {
        lema.to_string()
    } else {
        format!("{lema}_{categoria}")
    }
}

fn extraer_atributo(features: &str, clave: &str) -> Option<String> {
    let prefijo = format!("{clave}:");

    for item in features.split('|') {
        if let Some(valor) = item.strip_prefix(&prefijo) {
            return Some(valor.to_string());
        }
    }

    None
}
