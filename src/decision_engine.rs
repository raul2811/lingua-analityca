use std::collections::HashSet;

use crate::models::{
    DecisionResult, Dependencia, FactorDecision, LinguakitResult, StanfordResult, TextProfile,
    TokenInfo,
};

const UMBRAL_GANADOR_CLARO: f32 = 15.0;
const UMBRAL_GANADOR_MEDIO: f32 = 8.0;
const UMBRAL_CALIDAD_MINIMA: f32 = 35.0;

pub fn decidir(
    texto: &str,
    tipo: &str,
    stanford: &StanfordResult,
    linguakit: &LinguakitResult,
) -> DecisionResult {
    let perfil_texto = perfilar_texto(texto, tipo);
    let mut factores = Vec::new();

    agregar_factor(
        &mut factores,
        "Tecnico",
        "Disponibilidad de ejecucion",
        10.0,
        score_estado(&stanford.estado),
        score_estado(&linguakit.estado),
        "Premia respuestas OK y penaliza errores HTTP, CLI o JSON; disponibilidad no equivale a calidad linguistica.",
    );

    agregar_factor(
        &mut factores,
        "Linguistico",
        "Tokenizacion y POS",
        12.0,
        score_tokens_stanford(&stanford.tokens_pos, perfil_texto.tokens_estimados),
        score_tokens_linguakit(&linguakit.tokens, perfil_texto.tokens_estimados),
        "Aproxima las dimensiones Tokens/Words/UPOS usadas en evaluacion UD-CoNLL.",
    );

    agregar_factor(
        &mut factores,
        "Linguistico",
        "Lematizacion util",
        14.0,
        score_lemmas_stanford(&stanford.tokens_pos),
        score_lemmas_linguakit(&linguakit.tokens),
        "Se inspira en BLEX: los lemas son evidencia lexica, especialmente en verbos flexionados del espanol.",
    );

    agregar_factor(
        &mut factores,
        "Linguistico",
        "Cobertura de dependencias",
        18.0,
        score_dependencias(
            stanford.dependencias.len(),
            stanford.tokens_pos.len(),
            perfil_texto.tokens_estimados,
        ),
        score_dependencias(
            linguakit.dependencias.len(),
            linguakit.tokens.len(),
            perfil_texto.tokens_estimados,
        ),
        "Aproxima UAS/LAS sin gold standard: no solo importa que existan dependencias, sino su cobertura frente a los tokens.",
    );

    agregar_factor(
        &mut factores,
        "Linguistico",
        "Relaciones sintacticas clave",
        12.0,
        score_relaciones_clave_stanford(&stanford.dependencias),
        score_relaciones_clave_linguakit(&linguakit.dependencias),
        "Normaliza relaciones hacia equivalentes UD como nsubj, obj, det, obl, advmod y amod.",
    );

    agregar_factor(
        &mut factores,
        "Linguistico",
        "Validez estructural normalizada",
        12.0,
        score_estructura(&stanford.dependencias),
        score_estructura(&linguakit.dependencias),
        "Penaliza salidas sin arcos, autodependencias o relaciones vacias; es una validacion estructural reducida.",
    );

    let acuerdo = score_acuerdo(&stanford.dependencias, &linguakit.dependencias);
    agregar_factor(
        &mut factores,
        "Academico",
        "Acuerdo entre herramientas",
        10.0,
        acuerdo,
        acuerdo,
        "El acuerdo normalizado entre parsers aumenta confianza; desacuerdos fuertes activan revision o uso combinado.",
    );

    let (perfil_stanford, perfil_linguakit, perfil_exp) = score_perfil_texto(
        &perfil_texto,
        estado_ok(&stanford.estado),
        estado_ok(&linguakit.estado),
    );
    agregar_factor(
        &mut factores,
        "Contexto",
        "Ajuste al perfil del texto",
        8.0,
        perfil_stanford,
        perfil_linguakit,
        &perfil_exp,
    );

    agregar_factor(
        &mut factores,
        "Tecnico",
        "Integracion y control local",
        4.0,
        score_operacion_stanford(&stanford.estado),
        score_operacion_linguakit(&linguakit.estado),
        "Valora ejecucion local, integracion programatica y reproducibilidad del entorno.",
    );

    let puntos_stanford = total_para(&factores, |f| f.stanford);
    let puntos_linguakit = total_para(&factores, |f| f.linguakit);
    let diferencia = (puntos_stanford - puntos_linguakit).abs();
    let ambas_fallan = !estado_ok(&stanford.estado) && !estado_ok(&linguakit.estado);
    let calidad_baja = puntos_stanford.max(puntos_linguakit) < UMBRAL_CALIDAD_MINIMA;
    let decision_cerrada_riesgosa =
        diferencia < UMBRAL_GANADOR_MEDIO || perfil_texto.candidato_ambiguo;
    let requiere_revision_manual = ambas_fallan || calidad_baja || decision_cerrada_riesgosa;
    let confianza = calcular_confianza(diferencia, requiere_revision_manual, calidad_baja);
    let recomendacion = recomendar(
        puntos_stanford,
        puntos_linguakit,
        diferencia,
        ambas_fallan,
        calidad_baja,
        perfil_texto.candidato_ambiguo,
    );
    let justificacion = justificar(
        &recomendacion,
        puntos_stanford,
        puntos_linguakit,
        diferencia,
        &confianza,
        &factores,
        &perfil_texto,
    );

    DecisionResult {
        recomendacion,
        justificacion,
        puntos_stanford,
        puntos_linguakit,
        diferencia,
        puntos_stanford_fmt: formato_puntos(puntos_stanford),
        puntos_linguakit_fmt: formato_puntos(puntos_linguakit),
        diferencia_fmt: formato_puntos(diferencia),
        confianza,
        requiere_revision_manual,
        perfil_texto,
        factores,
        metodologia: "Matriz ponderada de 100 puntos inspirada en Universal Dependencies, CoNLL-U, LAS/MLAS/BLEX e indicadores tecnicos de reproducibilidad.".to_string(),
    }
}

fn perfilar_texto(texto: &str, tipo_usuario: &str) -> TextProfile {
    let tokens_estimados = texto
        .split_whitespace()
        .filter(|t| !t.trim().is_empty())
        .count();
    let es_pregunta = texto.contains('?') || texto.contains('¿');
    let tiene_ruido = detectar_ruido(texto);
    let candidato_ambiguo = detectar_ambiguedad(texto) || tipo_usuario == "ambiguo";
    let es_largo = tokens_estimados >= 15 || tipo_usuario == "largo";
    let tipo_detectado = if candidato_ambiguo {
        "ambiguo"
    } else if tiene_ruido || tipo_usuario == "informal" {
        "informal"
    } else if es_largo {
        "largo"
    } else {
        "formal"
    };

    TextProfile {
        tokens_estimados,
        es_largo,
        es_pregunta,
        tiene_ruido,
        candidato_ambiguo,
        tipo_usuario: tipo_usuario.to_string(),
        tipo_detectado: tipo_detectado.to_string(),
    }
}

fn detectar_ruido(texto: &str) -> bool {
    let lower = texto.to_lowercase();
    let marcas_sociales = ["http://", "https://", "@", "#", "jaja", "jeje", "xd"];
    let repeticion_puntuacion =
        texto.contains("!!") || texto.contains("??") || texto.contains("...");

    marcas_sociales.iter().any(|m| lower.contains(m)) || repeticion_puntuacion
}

fn detectar_ambiguedad(texto: &str) -> bool {
    let lower = texto.to_lowercase();
    lower.contains(" con el ")
        || lower.contains(" con la ")
        || lower.contains(" en el ")
        || lower.contains(" en la ")
        || lower.contains(" de el ")
        || lower.contains(" de la ")
}

fn agregar_factor(
    factores: &mut Vec<FactorDecision>,
    grupo: &str,
    criterio: &str,
    peso: f32,
    stanford_norm: f32,
    linguakit_norm: f32,
    explicacion: &str,
) {
    let stanford = redondear(peso * limitar(stanford_norm));
    let linguakit = redondear(peso * limitar(linguakit_norm));

    factores.push(FactorDecision {
        grupo: grupo.to_string(),
        criterio: criterio.to_string(),
        peso,
        stanford,
        linguakit,
        peso_fmt: formato_puntos(peso),
        stanford_fmt: formato_puntos(stanford),
        linguakit_fmt: formato_puntos(linguakit),
        explicacion: explicacion.to_string(),
    });
}

fn estado_ok(estado: &str) -> bool {
    estado.starts_with("OK")
}

fn score_estado(estado: &str) -> f32 {
    if estado_ok(estado) {
        1.0
    } else if estado.contains("JSON") {
        0.25
    } else {
        0.0
    }
}

fn score_tokens_stanford(tokens_pos: &[String], esperados: usize) -> f32 {
    if tokens_pos.is_empty() {
        return 0.0;
    }

    let cobertura = cobertura(tokens_pos.len(), esperados);
    let con_pos = tokens_pos
        .iter()
        .filter(|item| parsear_token_stanford(item).is_some_and(|(_, _, pos)| !pos.is_empty()))
        .count();
    let calidad_pos = con_pos as f32 / tokens_pos.len() as f32;

    0.55 * cobertura + 0.45 * calidad_pos
}

fn score_tokens_linguakit(tokens: &[TokenInfo], esperados: usize) -> f32 {
    if tokens.is_empty() {
        return 0.0;
    }

    let cobertura = cobertura(tokens.len(), esperados);
    let con_pos = tokens
        .iter()
        .filter(|t| !t.categoria.trim().is_empty() && t.categoria != "N/D")
        .count();
    let calidad_pos = con_pos as f32 / tokens.len() as f32;

    0.55 * cobertura + 0.45 * calidad_pos
}

fn score_lemmas_stanford(tokens_pos: &[String]) -> f32 {
    if tokens_pos.is_empty() {
        return 0.0;
    }

    let utiles = tokens_pos
        .iter()
        .filter_map(|item| parsear_token_stanford(item))
        .filter(|(token, lemma, _)| lema_util(token, lemma))
        .count();

    utiles as f32 / tokens_pos.len() as f32
}

fn score_lemmas_linguakit(tokens: &[TokenInfo]) -> f32 {
    if tokens.is_empty() {
        return 0.0;
    }

    let utiles = tokens
        .iter()
        .filter(|t| lema_util(&t.token, &t.lema))
        .count();

    utiles as f32 / tokens.len() as f32
}

fn lema_util(token: &str, lemma: &str) -> bool {
    let token_norm = token.to_lowercase();
    let lemma_norm = lemma.to_lowercase();

    !lemma_norm.is_empty()
        && lemma_norm != "n/d"
        && lemma_norm != "_"
        && (lemma_norm != token_norm || token_norm.len() > 6)
}

fn score_dependencias(deps: usize, tokens_reales: usize, tokens_estimados: usize) -> f32 {
    if deps == 0 {
        return 0.0;
    }

    let base = tokens_reales.max(tokens_estimados).max(1);
    let esperado = base.saturating_sub(1).max(1);
    (deps.min(esperado) as f32 / esperado as f32).min(1.0)
}

fn score_relaciones_clave_stanford(deps: &[Dependencia]) -> f32 {
    score_relaciones_clave(deps.iter().map(|d| d.relacion.as_str()))
}

fn score_relaciones_clave_linguakit(deps: &[Dependencia]) -> f32 {
    score_relaciones_clave(deps.iter().map(|d| map_linguakit_to_ud(&d.relacion)))
}

fn score_relaciones_clave<'a>(rels: impl Iterator<Item = &'a str>) -> f32 {
    let claves = [
        "nsubj", "obj", "iobj", "obl", "advmod", "amod", "det", "case",
    ];
    let encontradas: HashSet<&str> = rels.filter(|rel| claves.contains(rel)).collect();

    encontradas.len().min(6) as f32 / 6.0
}

fn score_estructura(deps: &[Dependencia]) -> f32 {
    if deps.is_empty() {
        return 0.0;
    }

    let relaciones_validas = deps
        .iter()
        .filter(|d| !d.relacion.trim().is_empty() && d.relacion != "dep")
        .count();
    let sin_auto_bucles = deps
        .iter()
        .filter(|d| {
            !d.gobernador.trim().is_empty()
                && !d.dependiente.trim().is_empty()
                && d.gobernador != d.dependiente
        })
        .count();

    let validez_rel = relaciones_validas as f32 / deps.len() as f32;
    let validez_arcos = sin_auto_bucles as f32 / deps.len() as f32;

    0.45 + (0.30 * validez_rel) + (0.25 * validez_arcos)
}

fn score_acuerdo(stanford: &[Dependencia], linguakit: &[Dependencia]) -> f32 {
    if stanford.is_empty() || linguakit.is_empty() {
        return 0.0;
    }

    let st: HashSet<String> = stanford
        .iter()
        .map(|d| arco_normalizado(&d.relacion, &d.dependiente))
        .collect();
    let lk: HashSet<String> = linguakit
        .iter()
        .map(|d| arco_normalizado(map_linguakit_to_ud(&d.relacion), &d.dependiente))
        .collect();
    let interseccion = st.intersection(&lk).count();
    let union = st.union(&lk).count().max(1);

    interseccion as f32 / union as f32
}

fn score_perfil_texto(
    perfil: &TextProfile,
    stanford_ok: bool,
    linguakit_ok: bool,
) -> (f32, f32, String) {
    let disponibilidad_stanford = if stanford_ok { 1.0 } else { 0.0 };
    let disponibilidad_linguakit = if linguakit_ok { 1.0 } else { 0.0 };

    if perfil.candidato_ambiguo {
        return (
            0.55 * disponibilidad_stanford,
            0.55 * disponibilidad_linguakit,
            "El texto contiene posibles adjuntos ambiguos o fue marcado como ambiguo; el motor reduce el sesgo y favorece revision combinada.".to_string(),
        );
    }

    match perfil.tipo_detectado.as_str() {
        "informal" => (
            0.55 * disponibilidad_stanford,
            0.85 * disponibilidad_linguakit,
            "El texto presenta rasgos informales o ruido; se prioriza lematizacion y analisis morfosintactico robusto en espanol.".to_string(),
        ),
        "largo" => (
            0.75 * disponibilidad_stanford,
            0.70 * disponibilidad_linguakit,
            "El texto es largo; se prioriza cobertura de dependencias, pero sin descartar la salida morfosintactica.".to_string(),
        ),
        _ => (
            0.80 * disponibilidad_stanford,
            0.70 * disponibilidad_linguakit,
            "El texto parece formal o editado; se favorece ligeramente la salida sintactica estructurada.".to_string(),
        ),
    }
}

fn score_operacion_stanford(estado: &str) -> f32 {
    if estado_ok(estado) { 0.95 } else { 0.0 }
}

fn score_operacion_linguakit(estado: &str) -> f32 {
    if estado.contains("OK LOCAL") {
        1.0
    } else if estado.contains("OK API") {
        0.85
    } else {
        0.0
    }
}

fn map_linguakit_to_ud(rel: &str) -> &str {
    if rel.starts_with("DobjPrep") || rel.starts_with("Dobj") {
        return "obj";
    }

    if rel.starts_with("Cprep") || rel.starts_with("Creg") || rel.starts_with("Adjn") {
        return "obl";
    }

    match rel {
        "SubjL" | "SubjR" => "nsubj",
        "IobjL" | "IobjR" => "iobj",
        "SpecL" | "SpecR" => "det",
        "AdjunctL" | "AdjunctR" => "advmod",
        "AtrL" | "AtrR" => "cop",
        "CoordL" | "CoordR" => "conj",
        "PrepL" | "PrepR" => "case",
        "ModL" | "ModR" => "amod",
        other => other,
    }
}

fn parsear_token_stanford(item: &str) -> Option<(&str, &str, &str)> {
    let mut partes = item.rsplitn(3, '_');
    let pos = partes.next()?;
    let lemma = partes.next()?;
    let token = partes.next()?;

    Some((token, lemma, pos))
}

fn cobertura(real: usize, esperado: usize) -> f32 {
    if esperado == 0 {
        return if real > 0 { 1.0 } else { 0.0 };
    }

    real.min(esperado) as f32 / esperado as f32
}

fn arco_normalizado(rel: &str, dependiente: &str) -> String {
    format!("{}:{}", rel.to_lowercase(), normalizar_nodo(dependiente))
}

fn normalizar_nodo(valor: &str) -> String {
    let base = valor.rsplitn(3, '_').last().unwrap_or(valor);
    normalizar_texto(base)
}

fn normalizar_texto(valor: &str) -> String {
    valor
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect()
}

fn total_para(factores: &[FactorDecision], f: impl Fn(&FactorDecision) -> f32) -> f32 {
    redondear(factores.iter().map(f).sum())
}

fn recomendar(
    puntos_stanford: f32,
    puntos_linguakit: f32,
    diferencia: f32,
    ambas_fallan: bool,
    calidad_baja: bool,
    candidato_ambiguo: bool,
) -> String {
    if ambas_fallan || calidad_baja {
        return "SALIDA INSUFICIENTE / REVISION MANUAL".to_string();
    }

    if diferencia < UMBRAL_GANADOR_MEDIO || candidato_ambiguo {
        return "USO COMBINADO / REVISION MANUAL".to_string();
    }

    if puntos_stanford > puntos_linguakit {
        if diferencia >= UMBRAL_GANADOR_CLARO {
            "USAR STANFORD CORENLP".to_string()
        } else {
            "PREFERIR STANFORD CON VALIDACION".to_string()
        }
    } else if diferencia >= UMBRAL_GANADOR_CLARO {
        "USAR LINGUAKIT LOCAL".to_string()
    } else {
        "PREFERIR LINGUAKIT CON VALIDACION".to_string()
    }
}

fn calcular_confianza(diferencia: f32, revision: bool, calidad_baja: bool) -> String {
    if calidad_baja {
        "Muy baja".to_string()
    } else if revision {
        "Baja".to_string()
    } else if diferencia >= UMBRAL_GANADOR_CLARO {
        "Alta".to_string()
    } else {
        "Media".to_string()
    }
}

fn justificar(
    recomendacion: &str,
    puntos_stanford: f32,
    puntos_linguakit: f32,
    diferencia: f32,
    confianza: &str,
    factores: &[FactorDecision],
    perfil: &TextProfile,
) -> String {
    let mut ventajas_stanford = Vec::new();
    let mut ventajas_linguakit = Vec::new();

    for factor in factores {
        let delta = factor.stanford - factor.linguakit;

        if delta >= 2.0 {
            ventajas_stanford.push(factor.criterio.as_str());
        } else if delta <= -2.0 {
            ventajas_linguakit.push(factor.criterio.as_str());
        }
    }

    let st = resumen_ventajas(&ventajas_stanford);
    let lk = resumen_ventajas(&ventajas_linguakit);

    format!(
        "{}. Stanford obtuvo {:.1}/100 y Linguakit {:.1}/100; diferencia {:.1} puntos, confianza {}. Perfil detectado: {} (usuario: {}, tokens estimados: {}). Fortalezas Stanford: {}. Fortalezas Linguakit: {}. La recomendacion usa una matriz ponderada, no una suma de presencia/ausencia.",
        recomendacion,
        puntos_stanford,
        puntos_linguakit,
        diferencia,
        confianza,
        perfil.tipo_detectado,
        perfil.tipo_usuario,
        perfil.tokens_estimados,
        st,
        lk,
    )
}

fn resumen_ventajas(ventajas: &[&str]) -> String {
    if ventajas.is_empty() {
        return "sin ventaja dominante".to_string();
    }

    ventajas
        .iter()
        .take(3)
        .copied()
        .collect::<Vec<_>>()
        .join(", ")
}

fn limitar(valor: f32) -> f32 {
    valor.clamp(0.0, 1.0)
}

fn redondear(valor: f32) -> f32 {
    (valor * 10.0).round() / 10.0
}

fn formato_puntos(valor: f32) -> String {
    format!("{valor:.1}")
}
