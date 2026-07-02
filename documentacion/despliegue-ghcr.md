# Despliegue con imagenes GHCR

Este flujo ejecuta los tres servicios desde imagenes publicadas en GitHub Container Registry.

## Imagenes

```text
ghcr.io/raul2811/lingua-analytica
ghcr.io/raul2811/lingua-analytica-stanford
ghcr.io/raul2811/lingua-analytica-linguakit
```

## Variables

```bash
export GHCR_IMAGE_PREFIX=ghcr.io/raul2811
export IMAGE_TAG=main
```

Para despliegues reproducibles se recomienda usar `sha-<git-sha>` en vez de `main`.

## Levantar el stack completo

Compose recomendado para produccion:

```bash
podman-compose -f compose.prod.yaml up -d
```

Tambien se mantiene `compose.yaml` como compose base del proyecto:

```bash
podman-compose up -d
```

Puertos publicados:

```text
Aplicacion Rust:  http://localhost:8080
Stanford CoreNLP: http://localhost:19000
Linguakit API:    http://localhost:3002
```

## Construccion automatica

El workflow `.github/workflows/build-images.yml` construye y publica las tres imagenes cuando cambian:

```text
Cargo.toml
Cargo.lock
src/**
templates/**
static/**
containers/**
.github/workflows/build-images.yml
```

La imagen de Rust se reconstruye cuando cambian codigo Rust, plantillas, frontend estatico o integracion con APIs. Las imagenes de Stanford y Linguakit se reconstruyen cuando cambian sus `Containerfile`.

## Origen de las tools

Las tools no se versionan en `tools/`:

```text
Linguakit: https://github.com/citiususc/Linguakit.git
Stanford:  https://nlp.stanford.edu/software/stanford-corenlp-4.5.10.zip
Modelo ES: https://nlp.stanford.edu/software/stanford-corenlp-4.5.10-models-spanish.jar
```

Esto mantiene el clon del repositorio liviano y mueve los artefactos pesados al build de imagen.
