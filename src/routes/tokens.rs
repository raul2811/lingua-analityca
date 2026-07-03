use std::collections::HashSet;

use crate::models::{LinguakitResult, MapeoDependencia, StanfordResult, TokenComparacion};

pub(super) fn comparar_tokens(
    stanford: &StanfordResult,
    linguakit: &LinguakitResult,
) -> Vec<TokenComparacion> {
    let st_tokens: Vec<(String, String, String)> = stanford
        .tokens_pos
        .iter()
        .filter_map(|item| parsear_token_stanford(item))
        .collect();
    let mut filas = Vec::new();
    let mut i = 0;
    let mut j = 0;

    while i < st_tokens.len() || j < linguakit.tokens.len() {
        let st = st_tokens.get(i);
        let lk = linguakit.tokens.get(j);

        if let Some((token, lema, pos)) = st {
            if token.eq_ignore_ascii_case("al")
                && linguakit.tokens.get(j).is_some_and(|t| t.token == "a")
                && linguakit.tokens.get(j + 1).is_some_and(|t| t.token == "el")
            {
                filas.push(TokenComparacion {
                    token: token.clone(),
                    stanford_lema: lema.clone(),
                    linguakit_lema: "a + el".to_string(),
                    stanford_pos: pos.clone(),
                    linguakit_pos: "PRP + DET".to_string(),
                    diferencia: "Contraccion alineada".to_string(),
                });
                i += 1;
                j += 2;
                continue;
            }

            if token.eq_ignore_ascii_case("del")
                && linguakit.tokens.get(j).is_some_and(|t| t.token == "de")
                && linguakit.tokens.get(j + 1).is_some_and(|t| t.token == "el")
            {
                filas.push(TokenComparacion {
                    token: token.clone(),
                    stanford_lema: lema.clone(),
                    linguakit_lema: "de + el".to_string(),
                    stanford_pos: pos.clone(),
                    linguakit_pos: "PRP + DET".to_string(),
                    diferencia: "Contraccion alineada".to_string(),
                });
                i += 1;
                j += 2;
                continue;
            }
        }

        let token = st
            .map(|(token, _, _)| token.clone())
            .or_else(|| lk.map(|token| token.token.clone()))
            .unwrap_or_else(|| "-".to_string());
        let stanford_lema = st
            .map(|(_, lema, _)| lema.clone())
            .unwrap_or_else(|| "-".to_string());
        let stanford_pos = st
            .map(|(_, _, pos)| pos.clone())
            .unwrap_or_else(|| "-".to_string());
        let linguakit_lema = lk
            .map(|token| token.lema.clone())
            .unwrap_or_else(|| "-".to_string());
        let linguakit_pos = lk
            .map(|token| token.categoria.clone())
            .unwrap_or_else(|| "-".to_string());

        filas.push(TokenComparacion {
            diferencia: describir_diferencia_token(
                &token,
                &stanford_lema,
                &linguakit_lema,
                &stanford_pos,
                &linguakit_pos,
            ),
            token,
            stanford_lema,
            linguakit_lema,
            stanford_pos,
            linguakit_pos,
        });

        i += usize::from(st.is_some());
        j += usize::from(lk.is_some());
    }

    filas
}

fn describir_diferencia_token(
    token: &str,
    stanford_lema: &str,
    linguakit_lema: &str,
    stanford_pos: &str,
    linguakit_pos: &str,
) -> String {
    if stanford_lema == "-" || linguakit_lema == "-" {
        return "Sin alineacion directa".to_string();
    }

    if stanford_lema == linguakit_lema && stanford_pos == linguakit_pos {
        return "Coincide".to_string();
    }

    let token_norm = token.to_lowercase();
    let linguakit_normaliza = linguakit_lema != token_norm && linguakit_lema != token;
    let stanford_no_normaliza = stanford_lema == token_norm || stanford_lema == token;

    if stanford_pos == linguakit_pos && linguakit_normaliza && stanford_no_normaliza {
        return "Linguakit lematiza mejor".to_string();
    }

    if stanford_pos == linguakit_pos {
        return "Diferencia de lema".to_string();
    }

    "Diferencia POS/lema".to_string()
}

pub(super) fn parsear_token_stanford(item: &str) -> Option<(String, String, String)> {
    let mut partes = item.rsplitn(3, '_');
    let pos = partes.next()?.to_string();
    let lema = partes.next()?.to_string();
    let token = partes.next()?.to_string();

    Some((token, lema, pos))
}

pub(super) fn mapear_dependencias_linguakit(linguakit: &LinguakitResult) -> Vec<MapeoDependencia> {
    let mut vistos = HashSet::new();
    let mut mapeos = Vec::new();

    for dep in &linguakit.dependencias {
        if !vistos.insert(dep.relacion.clone()) {
            continue;
        }

        let ud = map_linguakit_to_ud(&dep.relacion).to_string();

        mapeos.push(MapeoDependencia {
            linguakit: dep.relacion.clone(),
            ud_aproximado: ud.clone(),
            significado: significado_ud(&ud).to_string(),
            expresion: format!("{} ≈ {}", dep.relacion, ud),
        });
    }

    mapeos
}

pub(super) fn map_linguakit_to_ud(rel: &str) -> &str {
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
        _ => "dep",
    }
}

fn significado_ud(rel: &str) -> &str {
    match rel {
        "nsubj" => "sujeto nominal",
        "obj" => "objeto directo",
        "iobj" => "objeto indirecto",
        "det" => "determinante",
        "obl" => "adjunto oblicuo o complemento preposicional",
        "advmod" => "modificador adverbial",
        "amod" => "modificador adjetival",
        "case" => "marca preposicional",
        "cop" => "cópula",
        "conj" => "conjunción",
        _ => "dependencia no normalizada",
    }
}
