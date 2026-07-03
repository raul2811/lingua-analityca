use crate::models::{AnalisisResultado, GoldEvaluation, ResumenEjecutivo, TokenComparacion};

pub(super) fn construir_resumen_ejecutivo(
    decision: &crate::models::DecisionResult,
    comparacion: &[TokenComparacion],
    gold: &GoldEvaluation,
    deps_stanford: usize,
    deps_linguakit: usize,
) -> ResumenEjecutivo {
    let mut puntos = Vec::new();
    let mut evidencia_clave = Vec::new();

    if deps_stanford > deps_linguakit {
        puntos
            .push("Stanford fue más fuerte en cobertura sintáctica y dependencias UD.".to_string());
    }

    if comparacion
        .iter()
        .any(|t| t.diferencia == "Linguakit lematiza mejor")
    {
        puntos.push(
            "Linguakit aportó mejores lemas en español para algunos tokens flexionados."
                .to_string(),
        );
    }

    if decision.requiere_revision_manual {
        puntos.push("Se recomienda uso combinado o revisión manual por baja diferencia, ambigüedad o incertidumbre estructural.".to_string());
    }

    if gold.disponible {
        puntos.push(format!(
            "Coincide con gold manual: Stanford tokens {}, Linguakit tokens {}, Stanford dependencias {}, Linguakit dependencias {}.",
            gold.stanford_token_score_fmt,
            gold.linguakit_token_score_fmt,
            gold.stanford_dep_score_fmt,
            gold.linguakit_dep_score_fmt
        ));
    }

    for item in comparacion
        .iter()
        .filter(|t| t.diferencia == "Linguakit lematiza mejor")
        .take(3)
    {
        evidencia_clave.push(format!(
            "{}: Stanford `{}` vs Linguakit `{}`",
            item.token, item.stanford_lema, item.linguakit_lema
        ));
    }

    if evidencia_clave.is_empty() {
        evidencia_clave.push(
            "No se detectó una diferencia léxica dominante en los tokens alineados.".to_string(),
        );
    }

    ResumenEjecutivo {
        puntos,
        evidencia_clave,
    }
}

pub(super) fn explicar_revision(decision: &crate::models::DecisionResult) -> Vec<String> {
    let mut razones = Vec::new();

    if decision.diferencia < 8.0 {
        razones.push(format!(
            "La diferencia es {} puntos, menor al umbral de 8 puntos para elegir ganador.",
            decision.diferencia_fmt
        ));
    }

    if decision.perfil_texto.candidato_ambiguo || decision.perfil_texto.tipo_detectado == "ambiguo"
    {
        razones.push("El perfil del texto fue marcado o detectado como ambiguo.".to_string());
    }

    if decision.confianza == "Baja" || decision.confianza == "Muy baja" {
        razones.push(format!("La confianza calculada es {}.", decision.confianza));
    }

    if decision.requiere_revision_manual {
        razones.push(
            "Ambas herramientas aportan evidencia útil, pero ninguna domina con margen suficiente."
                .to_string(),
        );
    }

    razones
}

pub(super) fn construir_limitaciones(
    gold: &GoldEvaluation,
    decision: &crate::models::DecisionResult,
    comparacion: &[TokenComparacion],
) -> Vec<String> {
    let mut limitaciones = vec![
        "El mapeo Linguakit → UD es aproximado; permite comparación funcional, no equivalencia formal completa.".to_string(),
        "LAS, MLAS y BLEX se documentan como referencia académica, pero no se calculan oficialmente sin corpus CoNLL-U gold completo.".to_string(),
        "La comparación token por token puede requerir alineación especial en contracciones como `al` o `del`.".to_string(),
    ];

    if !gold.disponible {
        limitaciones.push("La oración actual no coincide con un caso gold manual; la evaluación queda como rúbrica heurística trazable.".to_string());
    }

    if decision.requiere_revision_manual {
        limitaciones.push("El sistema recomienda revisión manual cuando hay ambigüedad, baja diferencia de puntaje o desacuerdo entre parsers.".to_string());
    }

    if comparacion
        .iter()
        .any(|t| t.diferencia == "Sin alineacion directa")
    {
        limitaciones.push("Hay tokens sin alineación directa entre herramientas; esto puede afectar la comparación fina de lemas/POS.".to_string());
    }

    limitaciones
}

pub(super) fn construir_reporte_markdown(r: &AnalisisResultado) -> String {
    let mut md = String::new();
    md.push_str("# Reporte Lingua-Analytica\n\n");
    md.push_str(&format!("Entrada: `{}`\n\n", r.entrada));
    md.push_str(&format!("Tipo seleccionado: `{}`\n\n", r.tipo));
    md.push_str(&format!(
        "Recomendacion: **{}**\n\n",
        r.decision.recomendacion
    ));
    md.push_str(&format!("Confianza: **{}**\n\n", r.decision.confianza));
    md.push_str(&format!(
        "Puntajes: Stanford {} / 100, Linguakit {} / 100, diferencia {} puntos.\n\n",
        r.decision.puntos_stanford_fmt, r.decision.puntos_linguakit_fmt, r.decision.diferencia_fmt
    ));
    md.push_str(&format!(
        "Estado: Stanford {} en {}, Linguakit {} en {}.\n\n",
        r.stanford_metricas.estado,
        r.stanford_metricas.latencia_fmt,
        r.linguakit_metricas.estado,
        r.linguakit_metricas.latencia_fmt
    ));

    md.push_str("## Resumen ejecutivo\n\n");
    for punto in &r.resumen_ejecutivo.puntos {
        md.push_str(&format!("- {punto}\n"));
    }
    md.push_str("\n### Evidencia clave\n\n");
    for evidencia in &r.resumen_ejecutivo.evidencia_clave {
        md.push_str(&format!("- {evidencia}\n"));
    }
    md.push('\n');

    if r.decision.requiere_revision_manual {
        md.push_str("## Por que no se elige un ganador unico\n\n");
        for razon in &r.razones_revision {
            md.push_str(&format!("- {razon}\n"));
        }
        md.push('\n');
    }

    md.push_str("## Evaluacion gold manual\n\n");
    if r.gold.disponible {
        md.push_str(&format!(
            "Caso: `{}`. {}\n\n",
            r.gold.caso, r.gold.descripcion
        ));
        md.push_str(&format!(
            "Stanford POS/Lema: {}. Linguakit POS/Lema: {}. Stanford dependencias: {}. Linguakit dependencias: {}.\n\n",
            r.gold.stanford_token_score_fmt,
            r.gold.linguakit_token_score_fmt,
            r.gold.stanford_dep_score_fmt,
            r.gold.linguakit_dep_score_fmt
        ));
    } else {
        md.push_str(&format!("{}\n\n", r.gold.nota));
    }

    md.push_str("## Factores\n\n");
    md.push_str("| Criterio | Peso | Stanford | Linguakit |\n");
    md.push_str("| --- | ---: | ---: | ---: |\n");

    for factor in &r.decision.factores {
        md.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            factor.criterio, factor.peso_fmt, factor.stanford_fmt, factor.linguakit_fmt
        ));
    }

    md.push_str("\n## Comparacion token por token\n\n");
    md.push_str(
        "| Token | Stanford lema | Linguakit lema | Stanford POS | Linguakit POS | Diferencia |\n",
    );
    md.push_str("| --- | --- | --- | --- | --- | --- |\n");

    for token in &r.comparacion_tokens {
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            token.token,
            token.stanford_lema,
            token.linguakit_lema,
            token.stanford_pos,
            token.linguakit_pos,
            token.diferencia
        ));
    }

    md.push_str("\n## Limitaciones\n\n");
    for limitacion in &r.limitaciones {
        md.push_str(&format!("- {limitacion}\n"));
    }

    md
}
