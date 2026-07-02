use crate::models::{DecisionResult, LinguakitResult, StanfordResult};

pub fn decidir(
    tipo: &str,
    stanford: &StanfordResult,
    linguakit: &LinguakitResult,
) -> DecisionResult {
    let mut puntos_stanford: u32 = 0;
    let mut puntos_linguakit: u32 = 0;

    if !stanford.dependencias.is_empty() {
        puntos_stanford += 30;
    }

    if !linguakit.dependencias.is_empty() {
        puntos_linguakit += 30;
    }

    if !stanford.tokens_pos.is_empty() {
        puntos_stanford += 15;
    }

    if !linguakit.tokens.is_empty() {
        puntos_linguakit += 15;
    }

    match tipo {
        "formal" => {
            puntos_stanford += 20;
            puntos_linguakit += 15;
        }
        "ambiguo" => {
            puntos_stanford += 25;
            puntos_linguakit += 15;
        }
        "largo" => {
            puntos_stanford += 20;
            puntos_linguakit += 20;
        }
        "informal" => {
            puntos_linguakit += 20;
            puntos_stanford += 10;
        }
        _ => {}
    }

    if stanford.estado.contains("OK REAL") {
        puntos_stanford += 15;
    }

    if linguakit.estado.contains("OK LOCAL") {
        puntos_linguakit += 15;
    }

    if puntos_stanford > puntos_linguakit {
        DecisionResult {
            recomendacion: "USAR STANFORD CORENLP".to_string(),
            justificacion: "Stanford obtiene mayor puntaje por su análisis sintáctico estructurado, su servidor local y su salida formal en dependencias gramaticales.".to_string(),
            puntos_stanford,
            puntos_linguakit,
        }
    } else if puntos_linguakit > puntos_stanford {
        DecisionResult {
            recomendacion: "USAR LINGUAKIT LOCAL".to_string(),
            justificacion: "Linguakit obtiene mayor puntaje porque se ejecuta localmente y entrega tokens, lemas, categorías gramaticales y dependencias sintácticas en español.".to_string(),
            puntos_stanford,
            puntos_linguakit,
        }
    } else {
        DecisionResult {
            recomendacion: "USO COMBINADO".to_string(),
            justificacion: "Ambas herramientas aportan valor. Se recomienda comparar sus salidas para justificar técnicamente la selección según la naturaleza del texto.".to_string(),
            puntos_stanford,
            puntos_linguakit,
        }
    }
}
