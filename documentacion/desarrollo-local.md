# Desarrollo local con Rust en el host

Este flujo es para desarrollar la aplicacion Rust sin ejecutar el contenedor de Rust. Solo se levantan las herramientas NLP como servicios.

## 1. Preparar variables

```bash
cp .env.example .env
```

Valores importantes:

```env
APP_HOST=127.0.0.1
APP_PORT=8080
STANFORD_URL=http://localhost:19000
LINGUAKIT_URL=http://localhost:3002
LINGUAKIT_MODE=dep
LINGUAKIT_OUTPUT=-a
```

`STANFORD_URL` usa `19000` porque el contenedor publica Stanford interno `9000` en el host como `19000`. Esto evita conflictos con servidores Stanford locales.

## 2. Levantar solo las tools

```bash
podman-compose -f compose.tools.yaml up -d
```

Servicios expuestos:

```text
Stanford CoreNLP: http://localhost:19000
Linguakit API:    http://localhost:3002
```

## 3. Ejecutar Rust local

```bash
cargo run
```

La aplicacion queda en:

```text
http://localhost:8080
```

## 4. Verificaciones rapidas

```bash
curl -sS http://localhost:3002/ping
curl -sS "http://localhost:8080/api/linguakit/local?texto=Hola%20mundo."
curl -sS "http://localhost:8080/api/stanford/local?texto=Hola%20mundo."
```

## 5. Detener servicios

```bash
podman-compose -f compose.tools.yaml down
```
