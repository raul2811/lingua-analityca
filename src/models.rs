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
pub struct DecisionResult {
    pub recomendacion: String,
    pub justificacion: String,
    pub puntos_stanford: u32,
    pub puntos_linguakit: u32,
}

#[derive(Debug, Serialize, Clone)]
pub struct AnalisisResultado {
    pub entrada: String,
    pub tipo: String,
    pub stanford: StanfordResult,
    pub linguakit: LinguakitResult,
    pub decision: DecisionResult,
}