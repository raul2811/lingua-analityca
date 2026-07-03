use crate::models::{
    Dependencia, GoldDependenciaEval, GoldEvaluation, GoldTokenEval, LinguakitResult,
    StanfordResult,
};

use super::tokens::{map_linguakit_to_ud, parsear_token_stanford};

#[derive(Clone)]
struct GoldCaso {
    entrada: &'static str,
    caso: &'static str,
    descripcion: &'static str,
    tokens: Vec<(&'static str, &'static str, &'static str)>,
    dependencias: Vec<(&'static str, &'static str, &'static str)>,
}

fn casos_gold() -> Vec<GoldCaso> {
    vec![
        GoldCaso {
            entrada: "El estudiante analizo la oracion correctamente.",
            caso: "oracion-simple",
            descripcion: "Oracion transitiva con sujeto, objeto directo y modificador adverbial.",
            tokens: vec![
                ("El", "el", "DET"),
                ("estudiante", "estudiante", "NOUN"),
                ("analizo", "analizar", "VERB"),
                ("la", "el", "DET"),
                ("oracion", "oracion", "NOUN"),
                ("correctamente", "correctamente", "ADV"),
            ],
            dependencias: vec![
                ("analizar", "nsubj", "estudiante"),
                ("estudiante", "det", "el"),
                ("analizar", "obj", "oracion"),
                ("oracion", "det", "el"),
                ("analizar", "advmod", "correctamente"),
            ],
        },
        GoldCaso {
            entrada: "Vi al hombre con el telescopio.",
            caso: "ambiguedad-preposicional",
            descripcion: "Caso ambiguo: el complemento preposicional puede modificar al verbo o al sustantivo.",
            tokens: vec![
                ("Vi", "ver", "VERB"),
                ("al", "a + el", "ADP"),
                ("hombre", "hombre", "NOUN"),
                ("con", "con", "ADP"),
                ("el", "el", "DET"),
                ("telescopio", "telescopio", "NOUN"),
            ],
            dependencias: vec![
                ("ver", "obj", "hombre"),
                ("telescopio", "case", "con"),
                ("telescopio", "det", "el"),
            ],
        },
        GoldCaso {
            entrada: "El informe tecnico describe la arquitectura del sistema.",
            caso: "texto-formal",
            descripcion: "Texto formal con sujeto nominal y objeto directo.",
            tokens: vec![
                ("El", "el", "DET"),
                ("informe", "informe", "NOUN"),
                ("tecnico", "tecnico", "ADJ"),
                ("describe", "describir", "VERB"),
                ("la", "el", "DET"),
                ("arquitectura", "arquitectura", "NOUN"),
                ("del", "de + el", "ADP"),
                ("sistema", "sistema", "NOUN"),
            ],
            dependencias: vec![
                ("describir", "nsubj", "informe"),
                ("informe", "det", "el"),
                ("informe", "amod", "tecnico"),
                ("describir", "obj", "arquitectura"),
            ],
        },
        GoldCaso {
            entrada: "El perro esta corriendo rapidamente.",
            caso: "progresivo-simple",
            descripcion: "Oracion con verbo principal y modificador adverbial.",
            tokens: vec![
                ("El", "el", "DET"),
                ("perro", "perro", "NOUN"),
                ("esta", "estar", "AUX"),
                ("corriendo", "correr", "VERB"),
                ("rapidamente", "rapidamente", "ADV"),
            ],
            dependencias: vec![
                ("correr", "nsubj", "perro"),
                ("perro", "det", "el"),
                ("correr", "advmod", "rapidamente"),
            ],
        },
        GoldCaso {
            entrada: "La profesora reviso el analisis sintactico.",
            caso: "objeto-directo",
            descripcion: "Oracion con determinantes, sujeto y objeto directo.",
            tokens: vec![
                ("La", "el", "DET"),
                ("profesora", "profesora", "NOUN"),
                ("reviso", "revisar", "VERB"),
                ("el", "el", "DET"),
                ("analisis", "analisis", "NOUN"),
                ("sintactico", "sintactico", "ADJ"),
            ],
            dependencias: vec![
                ("revisar", "nsubj", "profesora"),
                ("profesora", "det", "el"),
                ("revisar", "obj", "analisis"),
                ("analisis", "amod", "sintactico"),
            ],
        },
    ]
}

pub(super) fn evaluar_gold(
    texto: &str,
    stanford: &StanfordResult,
    linguakit: &LinguakitResult,
) -> GoldEvaluation {
    let Some(caso) = casos_gold()
        .into_iter()
        .find(|caso| normalizar_oracion(caso.entrada) == normalizar_oracion(texto))
    else {
        return GoldEvaluation {
            disponible: false,
            caso: "sin-gold".to_string(),
            descripcion: "La entrada no coincide con los casos gold incluidos.".to_string(),
            tokens: vec![],
            dependencias: vec![],
            stanford_token_score_fmt: "N/D".to_string(),
            linguakit_token_score_fmt: "N/D".to_string(),
            stanford_dep_score_fmt: "N/D".to_string(),
            linguakit_dep_score_fmt: "N/D".to_string(),
            nota: "No se calcula LAS real; se usa evaluacion reducida cuando la entrada coincide con casos anotados manualmente.".to_string(),
        };
    };

    let st_tokens: Vec<(String, String, String)> = stanford
        .tokens_pos
        .iter()
        .filter_map(|item| parsear_token_stanford(item))
        .collect();

    let mut st_idx = 0;
    let mut lk_idx = 0;
    let token_evals: Vec<GoldTokenEval> = caso
        .tokens
        .iter()
        .map(|(token, lema_esperado, pos_esperado)| {
            let st = st_tokens.get(st_idx);
            let (lk_lema, lk_pos, lk_consumidos) =
                token_linguakit_alineado(token, lk_idx, linguakit);
            let st_lema = st
                .map(|(_, lema, _)| lema.clone())
                .unwrap_or_else(|| "-".to_string());
            let st_pos = st
                .map(|(_, _, pos)| pos.clone())
                .unwrap_or_else(|| "-".to_string());

            st_idx += 1;
            lk_idx += lk_consumidos;

            GoldTokenEval {
                token: (*token).to_string(),
                lema_esperado: (*lema_esperado).to_string(),
                pos_esperado: (*pos_esperado).to_string(),
                stanford_ok: normalizar_texto_simple(&st_lema)
                    == normalizar_texto_simple(lema_esperado)
                    && pos_equivalente(&st_pos, pos_esperado),
                linguakit_ok: normalizar_texto_simple(&lk_lema)
                    == normalizar_texto_simple(lema_esperado)
                    && pos_equivalente(&lk_pos, pos_esperado),
                stanford_lema: st_lema,
                stanford_pos: st_pos,
                linguakit_lema: lk_lema,
                linguakit_pos: lk_pos,
            }
        })
        .collect();

    let dep_evals: Vec<GoldDependenciaEval> = caso
        .dependencias
        .iter()
        .map(|(gobernador, relacion, dependiente)| GoldDependenciaEval {
            gobernador: (*gobernador).to_string(),
            relacion: (*relacion).to_string(),
            dependiente: (*dependiente).to_string(),
            stanford_ok: contiene_dependencia(
                &stanford.dependencias,
                gobernador,
                relacion,
                dependiente,
                false,
            ),
            linguakit_ok: contiene_dependencia(
                &linguakit.dependencias,
                gobernador,
                relacion,
                dependiente,
                true,
            ),
        })
        .collect();

    GoldEvaluation {
        disponible: true,
        caso: caso.caso.to_string(),
        descripcion: caso.descripcion.to_string(),
        stanford_token_score_fmt: porcentaje_fmt(token_evals.iter().filter(|t| t.stanford_ok).count(), token_evals.len()),
        linguakit_token_score_fmt: porcentaje_fmt(token_evals.iter().filter(|t| t.linguakit_ok).count(), token_evals.len()),
        stanford_dep_score_fmt: porcentaje_fmt(dep_evals.iter().filter(|d| d.stanford_ok).count(), dep_evals.len()),
        linguakit_dep_score_fmt: porcentaje_fmt(dep_evals.iter().filter(|d| d.linguakit_ok).count(), dep_evals.len()),
        tokens: token_evals,
        dependencias: dep_evals,
        nota: "Evaluacion reducida: compara lemas, POS y dependencias esperadas en casos manuales; no sustituye LAS/MLAS/BLEX oficial con corpus CoNLL-U.".to_string(),
    }
}

fn contiene_dependencia(
    deps: &[Dependencia],
    gobernador: &str,
    relacion: &str,
    dependiente: &str,
    linguakit: bool,
) -> bool {
    deps.iter().any(|dep| {
        let rel = if linguakit {
            map_linguakit_to_ud(&dep.relacion)
        } else {
            dep.relacion.as_str()
        };

        rel == relacion
            && normalizar_texto_simple(&dep.gobernador)
                .contains(&normalizar_texto_simple(gobernador))
            && normalizar_texto_simple(&dep.dependiente)
                .contains(&normalizar_texto_simple(dependiente))
    })
}

fn pos_equivalente(pos: &str, esperado: &str) -> bool {
    pos == esperado
        || (esperado == "ADP" && pos.contains("PRP"))
        || (esperado == "ADP" && pos == "PRP")
        || (esperado == "PUNCT" && pos == "SENT")
        || (esperado == "AUX" && pos == "VERB")
}

fn token_linguakit_alineado(
    gold_token: &str,
    idx: usize,
    linguakit: &LinguakitResult,
) -> (String, String, usize) {
    let actual = linguakit.tokens.get(idx);
    let siguiente = linguakit.tokens.get(idx + 1);

    if gold_token.eq_ignore_ascii_case("al")
        && actual.is_some_and(|t| t.token.eq_ignore_ascii_case("a"))
        && siguiente.is_some_and(|t| t.token.eq_ignore_ascii_case("el"))
    {
        return ("a + el".to_string(), "PRP + DET".to_string(), 2);
    }

    if gold_token.eq_ignore_ascii_case("del")
        && actual.is_some_and(|t| t.token.eq_ignore_ascii_case("de"))
        && siguiente.is_some_and(|t| t.token.eq_ignore_ascii_case("el"))
    {
        return ("de + el".to_string(), "PRP + DET".to_string(), 2);
    }

    match actual {
        Some(token) => (token.lema.clone(), token.categoria.clone(), 1),
        None => ("-".to_string(), "-".to_string(), 1),
    }
}

fn normalizar_oracion(texto: &str) -> String {
    texto
        .trim()
        .trim_end_matches('.')
        .to_lowercase()
        .split_whitespace()
        .map(normalizar_texto_simple)
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalizar_texto_simple(texto: &str) -> String {
    texto
        .to_lowercase()
        .chars()
        .filter_map(|c| match c {
            'á' | 'à' | 'ä' | 'â' => Some('a'),
            'é' | 'è' | 'ë' | 'ê' => Some('e'),
            'í' | 'ì' | 'ï' | 'î' => Some('i'),
            'ó' | 'ò' | 'ö' | 'ô' => Some('o'),
            'ú' | 'ù' | 'ü' | 'û' => Some('u'),
            'ñ' => Some('n'),
            c if c.is_alphanumeric() => Some(c),
            _ => None,
        })
        .collect()
}

fn porcentaje_fmt(ok: usize, total: usize) -> String {
    if total == 0 {
        return "N/D".to_string();
    }

    format!("{:.1}%", (ok as f32 / total as f32) * 100.0)
}
