---
applyTo: "src/**,Cargo.toml,Cargo.lock"
---

Revisa Rust con criterios estrictos:

- `cargo fmt` obligatorio.
- `cargo clippy` sin warnings.
- `cargo test` obligatorio.
- No `unwrap()` ni `expect()` en producción salvo justificación.
- Clientes HTTP con timeout explícito.
- Errores tipados.
- No filtrar detalles internos en respuestas HTTP.
- Agregar tests cuando cambie lógica de decisión, parsing o scoring.
