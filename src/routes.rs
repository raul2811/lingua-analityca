use actix_web::{HttpResponse, Responder, get, post, web};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::time::Instant;
use tera::{Context, Tera};

use crate::decision_engine::decidir;
use crate::linguakit_client::analizar_linguakit_local;
use crate::models::{
    AnalisisForm, AnalisisResultado, AristaGrafoDependencia, Dependencia, GoldDependenciaEval,
    GoldEvaluation, GoldTokenEval, GrafoDependencia, LineaArbolDependencia, LinguakitResult,
    MapeoDependencia, NodoGrafoDependencia, ResumenEjecutivo, StanfordResult, TokenComparacion,
    ToolMetrics,
};
use crate::stanford_client::analizar_stanford_real;

#[derive(Debug, Deserialize)]
pub struct TextoQuery {
    pub texto: String,
}

#[get("/")]
pub async fn index(tmpl: web::Data<Tera>) -> impl Responder {
    let ctx = Context::new();

    match tmpl.render("index.html", &ctx) {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),

        Err(err) => HttpResponse::InternalServerError()
            .content_type("text/plain")
            .body(format!("Error renderizando index.html: {err}")),
    }
}

#[get("/metodologia")]
pub async fn metodologia(tmpl: web::Data<Tera>) -> impl Responder {
    let ctx = Context::new();

    match tmpl.render("metodologia.html", &ctx) {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),

        Err(err) => HttpResponse::InternalServerError()
            .content_type("text/plain")
            .body(format!("Error renderizando metodologia.html: {err}")),
    }
}

#[post("/analizar")]
pub async fn analizar(form: web::Form<AnalisisForm>, tmpl: web::Data<Tera>) -> impl Responder {
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

    let resultado = ejecutar_analisis(texto, &form.tipo).await;

    let mut ctx = Context::new();
    ctx.insert("r", &resultado);

    match tmpl.render("partials/resultado.html", &ctx) {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),

        Err(err) => HttpResponse::InternalServerError()
            .content_type("text/plain")
            .body(format!("Error renderizando resultado.html: {err}")),
    }
}

#[post("/api/analizar")]
pub async fn analizar_json(form: web::Form<AnalisisForm>) -> impl Responder {
    let texto = form.texto.trim();

    if texto.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "ok": false,
            "error": "La entrada de texto no puede estar vacía."
        }));
    }

    HttpResponse::Ok().json(ejecutar_analisis(texto, &form.tipo).await)
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

async fn ejecutar_analisis(texto: &str, tipo: &str) -> AnalisisResultado {
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

fn comparar_tokens(
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

fn parsear_token_stanford(item: &str) -> Option<(String, String, String)> {
    let mut partes = item.rsplitn(3, '_');
    let pos = partes.next()?.to_string();
    let lema = partes.next()?.to_string();
    let token = partes.next()?.to_string();

    Some((token, lema, pos))
}

fn mapear_dependencias_linguakit(linguakit: &LinguakitResult) -> Vec<MapeoDependencia> {
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

fn construir_arbol(deps: &[Dependencia]) -> Vec<LineaArbolDependencia> {
    if deps.is_empty() {
        return vec![LineaArbolDependencia {
            indent_px: 0,
            texto: "Sin dependencias disponibles".to_string(),
        }];
    }

    let mut hijos: HashMap<String, Vec<&Dependencia>> = HashMap::new();
    let mut gobernadores = HashSet::new();
    let mut dependientes = HashSet::new();

    for dep in deps {
        gobernadores.insert(dep.gobernador.clone());
        dependientes.insert(dep.dependiente.clone());
        hijos.entry(dep.gobernador.clone()).or_default().push(dep);
    }

    let mut raices: Vec<String> = gobernadores.difference(&dependientes).cloned().collect();
    raices.sort();

    if raices.is_empty() {
        raices = gobernadores.into_iter().collect();
        raices.sort();
    }

    let mut lineas = Vec::new();
    let mut visitados = HashSet::new();

    for raiz in raices.into_iter().take(3) {
        recorrer_arbol(&raiz, &raiz, 0, &hijos, &mut visitados, &mut lineas);
    }

    lineas
}

fn construir_grafo_dependencias(deps: &[Dependencia]) -> GrafoDependencia {
    if deps.is_empty() {
        return GrafoDependencia {
            width: 360,
            height: 140,
            nodos: vec![NodoGrafoDependencia {
                id: "sin-dependencias".to_string(),
                texto: "Sin dependencias disponibles".to_string(),
                x: 40,
                y: 50,
            }],
            aristas: vec![],
        };
    }

    let mut hijos: HashMap<String, Vec<&Dependencia>> = HashMap::new();
    let mut gobernadores = HashSet::new();
    let mut dependientes = HashSet::new();

    for dep in deps {
        gobernadores.insert(dep.gobernador.clone());
        dependientes.insert(dep.dependiente.clone());
        hijos.entry(dep.gobernador.clone()).or_default().push(dep);
    }

    let mut raices: Vec<String> = gobernadores.difference(&dependientes).cloned().collect();
    raices.sort();

    if raices.is_empty() {
        raices = gobernadores.into_iter().collect();
        raices.sort();
    }

    let mut posiciones: HashMap<String, (usize, usize)> = HashMap::new();
    let mut orden = 0;
    let mut visitados = HashSet::new();

    for raiz in raices {
        asignar_posiciones_grafo(
            &raiz,
            0,
            &hijos,
            &mut visitados,
            &mut posiciones,
            &mut orden,
        );
    }

    let mut nodos: Vec<NodoGrafoDependencia> = posiciones
        .iter()
        .map(|(texto, (x, y))| NodoGrafoDependencia {
            id: id_nodo(texto),
            texto: texto.clone(),
            x: *x,
            y: *y,
        })
        .collect();
    nodos.sort_by_key(|n| (n.y, n.x));

    let aristas: Vec<AristaGrafoDependencia> = deps
        .iter()
        .filter_map(|dep| {
            let (x1, y1) = *posiciones.get(&dep.gobernador)?;
            let (x2, y2) = *posiciones.get(&dep.dependiente)?;

            Some(AristaGrafoDependencia {
                desde: id_nodo(&dep.gobernador),
                hacia: id_nodo(&dep.dependiente),
                relacion: dep.relacion.clone(),
                x1: x1 + 70,
                y1: y1 + 34,
                x2: x2 + 70,
                y2: y2,
                label_x: (x1 + x2) / 2 + 48,
                label_y: (y1 + y2) / 2 + 8,
                ud_aproximado: map_linguakit_to_ud(&dep.relacion).to_string(),
                etiqueta_visual: etiqueta_arista(&dep.relacion),
            })
        })
        .collect();

    let width = nodos.iter().map(|n| n.x).max().unwrap_or(360) + 190;
    let height = nodos.iter().map(|n| n.y).max().unwrap_or(140) + 100;

    GrafoDependencia {
        width,
        height,
        nodos,
        aristas,
    }
}

fn asignar_posiciones_grafo(
    nodo: &str,
    profundidad: usize,
    hijos: &HashMap<String, Vec<&Dependencia>>,
    visitados: &mut HashSet<String>,
    posiciones: &mut HashMap<String, (usize, usize)>,
    orden: &mut usize,
) {
    if !visitados.insert(nodo.to_string()) {
        return;
    }

    let x = 40 + (*orden * 165);
    let y = 40 + (profundidad * 92);
    posiciones.insert(nodo.to_string(), (x, y));
    *orden += 1;

    if let Some(deps) = hijos.get(nodo) {
        for dep in deps {
            asignar_posiciones_grafo(
                &dep.dependiente,
                profundidad + 1,
                hijos,
                visitados,
                posiciones,
                orden,
            );
        }
    }
}

fn id_nodo(texto: &str) -> String {
    texto
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect()
}

fn etiqueta_arista(relacion: &str) -> String {
    let ud = map_linguakit_to_ud(relacion);

    if ud != "dep" && ud != relacion {
        format!("{relacion} ≈ {ud}")
    } else {
        relacion.to_string()
    }
}

fn recorrer_arbol(
    nodo: &str,
    texto: &str,
    nivel: usize,
    hijos: &HashMap<String, Vec<&Dependencia>>,
    visitados: &mut HashSet<String>,
    lineas: &mut Vec<LineaArbolDependencia>,
) {
    if !visitados.insert(nodo.to_string()) {
        return;
    }

    lineas.push(LineaArbolDependencia {
        indent_px: nivel * 20,
        texto: texto.to_string(),
    });

    if let Some(deps) = hijos.get(nodo) {
        for dep in deps {
            let texto = format!("{} [{}]", dep.dependiente, dep.relacion);
            recorrer_arbol(
                &dep.dependiente,
                &texto,
                nivel + 1,
                hijos,
                visitados,
                lineas,
            );
        }
    }
}

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

fn evaluar_gold(
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

fn construir_resumen_ejecutivo(
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

fn explicar_revision(decision: &crate::models::DecisionResult) -> Vec<String> {
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

fn construir_limitaciones(
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

fn construir_reporte_markdown(r: &AnalisisResultado) -> String {
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
