mod analysis;
mod batch;
mod gold;
mod graph;
mod handlers;
mod report;
mod tokens;

pub use handlers::{
    analizar, analizar_json, analizar_lote_json, health, index, metodologia,
    probar_linguakit_local, probar_stanford_local,
};
