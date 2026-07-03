use std::time::Instant;

use crate::decision_engine::decidir;
use crate::linguakit_client::analizar_linguakit_local;
use crate::models::{AnalisisResultado, ToolMetrics};
use crate::stanford_client::analizar_stanford_real;

use super::gold::evaluar_gold;
use super::graph::{construir_arbol, construir_grafo_dependencias};
use super::report::{
    construir_limitaciones, construir_reporte_markdown, construir_resumen_ejecutivo,
    explicar_revision,
};
use super::tokens::{comparar_tokens, mapear_dependencias_linguakit};

pub(super) async fn ejecutar_analisis(texto: &str, tipo: &str) -> AnalisisResultado {
    let inicio_stanford = Instant::now();
    let stanford = analizar_stanford_real(texto).await;
    let stanford_ms = inicio_stanford.elapsed().as_millis();

    let inicio_linguakit = Instant::now();
    let linguakit = analizar_linguakit_local(texto).await;
    let linguakit_ms = inicio_linguakit.elapsed().as_millis();

    let decision = decidir(texto, tipo, &stanford, &linguakit);
    let comparacion_tokens = comparar_tokens(&stanford, &linguakit);
    let mapeo_dependencias = mapear_dependencias_linguakit(&linguakit);
    let arbol_stanford = construir_arbol(&stanford.dependencias);
    let arbol_linguakit = construir_arbol(&linguakit.dependencias);
    let grafo_stanford = construir_grafo_dependencias(&stanford.dependencias);
    let grafo_linguakit = construir_grafo_dependencias(&linguakit.dependencias);
    let stanford_metricas = metricas_herramienta(&stanford.estado, stanford_ms);
    let linguakit_metricas = metricas_herramienta(&linguakit.estado, linguakit_ms);
    let gold = evaluar_gold(texto, &stanford, &linguakit);
    let resumen_ejecutivo = construir_resumen_ejecutivo(
        &decision,
        &comparacion_tokens,
        &gold,
        stanford.dependencias.len(),
        linguakit.dependencias.len(),
    );
    let razones_revision = explicar_revision(&decision);
    let limitaciones = construir_limitaciones(&gold, &decision, &comparacion_tokens);

    let mut resultado = AnalisisResultado {
        entrada: texto.to_string(),
        tipo: tipo.to_string(),
        stanford,
        linguakit,
        decision,
        stanford_metricas,
        linguakit_metricas,
        comparacion_tokens,
        mapeo_dependencias,
        arbol_stanford,
        arbol_linguakit,
        grafo_stanford,
        grafo_linguakit,
        gold,
        resumen_ejecutivo,
        razones_revision,
        limitaciones,
        reporte_markdown: String::new(),
    };

    resultado.reporte_markdown = construir_reporte_markdown(&resultado);
    resultado
}

fn metricas_herramienta(estado: &str, latencia_ms: u128) -> ToolMetrics {
    ToolMetrics {
        estado: estado.to_string(),
        latencia_ms,
        latencia_fmt: format!("{latencia_ms} ms"),
    }
}
