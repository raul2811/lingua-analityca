---
applyTo: "containers/**,compose*.yaml,.dockerignore"
---

Revisa este cambio contra buenas prácticas de contenedores:

- Imagen base mínima.
- Multi-stage build.
- No usar `latest`.
- No ejecutar como root.
- No hardcodear secretos.
- Usar `COPY` sobre `ADD` salvo razón clara.
- Mantener `.dockerignore` actualizado.
- Usar `read_only`, `tmpfs`, `no-new-privileges` y `cap_drop: ALL`.
- Agregar `healthcheck` cuando aplique.
- No publicar puertos innecesarios.
- Preferir tags por SHA para despliegue.
