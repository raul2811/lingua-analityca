# Instrucciones para GitHub Copilot

Responde siempre en español.

Este repositorio sigue esta prioridad:

seguridad > corrección > mantenibilidad > rendimiento > velocidad de entrega

Revisa cada cambio como arquitecto DevSecOps senior.

Para Rust:
- No aceptar `unwrap()`, `expect()` o `panic!` en rutas productivas salvo justificación.
- Exigir `cargo fmt`, `cargo clippy` sin warnings y `cargo test`.
- Validar manejo de errores con `Result<T, E>`.
- Revisar timeouts en clientes HTTP.
- Evitar exposición de errores internos al usuario.
- Preferir funciones pequeñas, puras y testeables.

Para contenedores:
- No aceptar imágenes con `latest`.
- Exigir usuario no-root.
- Exigir `read_only` cuando sea posible.
- Exigir `cap_drop: ALL`.
- Exigir `no-new-privileges`.
- Exigir `healthcheck`.
- Exigir Trivy sin vulnerabilidades `HIGH` o `CRITICAL`.
- Exigir SBOM y firma de imágenes para releases.

Para este proyecto:
- La app Rust orquesta Stanford CoreNLP y Linguakit como servicios locales.
- No introducir dependencias externas en runtime si rompe el modo offline/local.
- Mantener la comparación NLP trazable y académicamente defendible.
- La lógica de decisión debe estar testeada.
