use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct AnalisisForm {
    pub texto: String,
    pub tipo: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct TokenInfo {
    pub token: String,
    pub lema: String,
    pub categoria: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct Dependencia {
    pub relacion: String,
    pub gobernador: String,
    pub dependiente: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct StanfordResult {
    pub estado: String,
    pub tokens_pos: Vec<String>,
    pub dependencias: Vec<Dependencia>,
    pub raw_json: serde_json::Value,
}

#[derive(Debug, Serialize, Clone)]
pub struct LinguakitResult {
    pub estado: String,
    pub tokens: Vec<TokenInfo>,
    pub dependencias: Vec<Dependencia>,
    pub raw_json: serde_json::Value,
}

#[derive(Debug, Serialize, Clone)]
pub struct TextProfile {
    pub tokens_estimados: usize,
    pub es_largo: bool,
    pub es_pregunta: bool,
    pub tiene_ruido: bool,
    pub candidato_ambiguo: bool,
    pub tipo_usuario: String,
    pub tipo_detectado: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct FactorDecision {
    pub grupo: String,
    pub criterio: String,
    pub peso: f32,
    pub stanford: f32,
    pub linguakit: f32,
    pub peso_fmt: String,
    pub stanford_fmt: String,
    pub linguakit_fmt: String,
    pub explicacion: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct DecisionResult {
    pub recomendacion: String,
    pub justificacion: String,
    pub puntos_stanford: f32,
    pub puntos_linguakit: f32,
    pub diferencia: f32,
    pub puntos_stanford_fmt: String,
    pub puntos_linguakit_fmt: String,
    pub diferencia_fmt: String,
    pub confianza: String,
    pub requiere_revision_manual: bool,
    pub perfil_texto: TextProfile,
    pub factores: Vec<FactorDecision>,
    pub metodologia: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ToolMetrics {
    pub estado: String,
    pub latencia_ms: u128,
    pub latencia_fmt: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct TokenComparacion {
    pub token: String,
    pub stanford_lema: String,
    pub linguakit_lema: String,
    pub stanford_pos: String,
    pub linguakit_pos: String,
    pub diferencia: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct MapeoDependencia {
    pub linguakit: String,
    pub ud_aproximado: String,
    pub significado: String,
    pub expresion: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct LineaArbolDependencia {
    pub indent_px: usize,
    pub texto: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct NodoGrafoDependencia {
    pub id: String,
    pub texto: String,
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Serialize, Clone)]
pub struct AristaGrafoDependencia {
    pub desde: String,
    pub hacia: String,
    pub relacion: String,
    pub x1: usize,
    pub y1: usize,
    pub x2: usize,
    pub y2: usize,
    pub label_x: usize,
    pub label_y: usize,
    pub ud_aproximado: String,
    pub etiqueta_visual: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct GrafoDependencia {
    pub width: usize,
    pub height: usize,
    pub nodos: Vec<NodoGrafoDependencia>,
    pub aristas: Vec<AristaGrafoDependencia>,
}

#[derive(Debug, Serialize, Clone)]
pub struct GoldTokenEval {
    pub token: String,
    pub lema_esperado: String,
    pub pos_esperado: String,
    pub stanford_lema: String,
    pub stanford_pos: String,
    pub stanford_ok: bool,
    pub linguakit_lema: String,
    pub linguakit_pos: String,
    pub linguakit_ok: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct GoldDependenciaEval {
    pub gobernador: String,
    pub relacion: String,
    pub dependiente: String,
    pub stanford_ok: bool,
    pub linguakit_ok: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct GoldEvaluation {
    pub disponible: bool,
    pub caso: String,
    pub descripcion: String,
    pub tokens: Vec<GoldTokenEval>,
    pub dependencias: Vec<GoldDependenciaEval>,
    pub stanford_token_score_fmt: String,
    pub linguakit_token_score_fmt: String,
    pub stanford_dep_score_fmt: String,
    pub linguakit_dep_score_fmt: String,
    pub nota: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ResumenEjecutivo {
    pub puntos: Vec<String>,
    pub evidencia_clave: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct AnalisisResultado {
    pub entrada: String,
    pub tipo: String,
    pub stanford: StanfordResult,
    pub linguakit: LinguakitResult,
    pub decision: DecisionResult,
    pub stanford_metricas: ToolMetrics,
    pub linguakit_metricas: ToolMetrics,
    pub comparacion_tokens: Vec<TokenComparacion>,
    pub mapeo_dependencias: Vec<MapeoDependencia>,
    pub arbol_stanford: Vec<LineaArbolDependencia>,
    pub arbol_linguakit: Vec<LineaArbolDependencia>,
    pub grafo_stanford: GrafoDependencia,
    pub grafo_linguakit: GrafoDependencia,
    pub gold: GoldEvaluation,
    pub resumen_ejecutivo: ResumenEjecutivo,
    pub razones_revision: Vec<String>,
    pub limitaciones: Vec<String>,
    pub reporte_markdown: String,
}
