---
applyTo: "**"
---

Revisa seguridad y supply chain:

- No hardcodear secretos, tokens, credenciales ni URLs sensibles.
- No ampliar permisos de GitHub Actions sin necesidad explícita.
- Preferir acciones y workflows pineados por versión estable o SHA.
- Mantener permisos mínimos en `permissions`.
- Bloquear hallazgos `HIGH` y `CRITICAL` en escaneo de imágenes y filesystem.
- Generar SBOM y firmar imágenes publicadas.
- Mantener el modo local/offline del proyecto.
