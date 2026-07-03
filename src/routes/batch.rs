use crate::models::{BatchAnalisisResultado, BatchChunkResultado, Dependencia};

use super::analysis::ejecutar_analisis;
use super::graph::construir_grafo_dependencias;

const BATCH_CHUNK_WORDS: usize = 350;

pub(super) async fn ejecutar_analisis_lote(
    texto: &str,
    tipo: &str,
) -> Result<BatchAnalisisResultado, String> {
    let total_palabras = contar_palabras(texto);

    if total_palabras == 0 {
        return Err("La entrada de texto no puede estar vacía.".to_string());
    }

    let fragmentos = dividir_en_fragmentos(texto, BATCH_CHUNK_WORDS);
    let mut chunks = Vec::with_capacity(fragmentos.len());
    let mut stanford_export = Vec::new();
    let mut linguakit_export = Vec::new();

    for (indice, fragmento) in fragmentos.iter().enumerate() {
        let resultado = ejecutar_analisis(fragmento, tipo).await;
        let chunk_id = indice + 1;

        stanford_export.extend(
            resultado
                .stanford
                .dependencias
                .iter()
                .map(|dep| prefijar_dependencia(dep, chunk_id)),
        );
        linguakit_export.extend(
            resultado
                .linguakit
                .dependencias
                .iter()
                .map(|dep| prefijar_dependencia(dep, chunk_id)),
        );

        chunks.push(BatchChunkResultado {
            indice: chunk_id,
            palabras: contar_palabras(fragmento),
            texto_preview: resumen_fragmento(fragmento),
            stanford_estado: resultado.stanford.estado,
            linguakit_estado: resultado.linguakit.estado,
            stanford_dependencias: resultado.stanford.dependencias,
            linguakit_dependencias: resultado.linguakit.dependencias,
            stanford_tokens: resultado.stanford.tokens_pos.len(),
            linguakit_tokens: resultado.linguakit.tokens.len(),
        });
    }

    let grafo_stanford = construir_grafo_dependencias(&stanford_export);
    let grafo_linguakit = construir_grafo_dependencias(&linguakit_export);
    let export_dot_stanford = construir_dot("Stanford CoreNLP", &stanford_export);
    let export_dot_linguakit = construir_dot("Linguakit", &linguakit_export);
    let stanford_dependencias_total = stanford_export.len();
    let linguakit_dependencias_total = linguakit_export.len();

    let export_json = serde_json::json!({
        "tipo": tipo,
        "total_palabras": total_palabras,
        "chunk_size": BATCH_CHUNK_WORDS,
        "total_chunks": chunks.len(),
        "chunks": chunks.clone(),
        "grafos": {
            "stanford": grafo_stanford,
            "linguakit": grafo_linguakit,
            "stanford_dot": export_dot_stanford,
            "linguakit_dot": export_dot_linguakit
        }
    });

    Ok(BatchAnalisisResultado {
        ok: true,
        tipo: tipo.to_string(),
        total_palabras,
        total_chunks: chunks.len(),
        chunk_size: BATCH_CHUNK_WORDS,
        chunks,
        stanford_dependencias_total,
        linguakit_dependencias_total,
        grafo_stanford,
        grafo_linguakit,
        export_json,
        export_dot_stanford,
        export_dot_linguakit,
        nota: "Analisis por lotes sin limite de palabras definido por la aplicacion: el documento se divide en fragmentos para mantener tiempos, memoria y grafos manejables.".to_string(),
    })
}

fn contar_palabras(texto: &str) -> usize {
    texto.split_whitespace().count()
}

fn dividir_en_fragmentos(texto: &str, chunk_words: usize) -> Vec<String> {
    let mut fragmentos = Vec::new();
    let mut actual = Vec::with_capacity(chunk_words);

    for palabra in texto.split_whitespace() {
        actual.push(palabra);

        if actual.len() >= chunk_words {
            fragmentos.push(actual.join(" "));
            actual.clear();
        }
    }

    if !actual.is_empty() {
        fragmentos.push(actual.join(" "));
    }

    fragmentos
}

fn resumen_fragmento(texto: &str) -> String {
    let mut preview = texto
        .split_whitespace()
        .take(24)
        .collect::<Vec<_>>()
        .join(" ");

    if contar_palabras(texto) > 24 {
        preview.push_str("...");
    }

    preview
}

fn prefijar_dependencia(dep: &Dependencia, chunk_id: usize) -> Dependencia {
    Dependencia {
        relacion: dep.relacion.clone(),
        gobernador: format!("c{chunk_id}:{}", dep.gobernador),
        dependiente: format!("c{chunk_id}:{}", dep.dependiente),
    }
}

fn construir_dot(nombre: &str, dependencias: &[Dependencia]) -> String {
    let mut dot = String::new();
    dot.push_str(&format!("digraph \"{}\" {{\n", escapar_dot(nombre)));
    dot.push_str("  graph [rankdir=TB, overlap=false, splines=true];\n");
    dot.push_str("  node [shape=box, style=\"rounded,filled\", fillcolor=\"#161f30\", fontcolor=\"#f8fafc\", color=\"#3b6ff5\"];\n");
    dot.push_str("  edge [fontname=\"monospace\", fontsize=10, color=\"#94a3b8\"];\n");

    for dep in dependencias {
        dot.push_str(&format!(
            "  \"{}\" -> \"{}\" [label=\"{}\"];\n",
            escapar_dot(&dep.gobernador),
            escapar_dot(&dep.dependiente),
            escapar_dot(&dep.relacion)
        ));
    }

    dot.push_str("}\n");
    dot
}

fn escapar_dot(valor: &str) -> String {
    valor.replace('\\', "\\\\").replace('"', "\\\"")
}
