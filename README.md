# 📊 LINGUA-ANALYTICA V2.1
### Herramientas de Análisis Sintáctico NLP & Matriz de Decisión Técnica

Prototipo académico desarrollado para el curso **Diseño de Compiladores (INF511)** en la **Universidad de Panamá**.

---

## 🚀 badges

<div align="center">

![Rust](https://img.shields.io/badge/Rust-Actix_Web-orange?style=for-the-badge&logo=rust)
![HTMX](https://img.shields.io/badge/HTMX-Frontend-blue?style=for-the-badge&logo=htmx)
![HTML CSS](https://img.shields.io/badge/HTML%2FCSS-UI-38B2AC?style=for-the-badge)
![NLP](https://img.shields.io/badge/NLP-Syntactic_Analysis-purple?style=for-the-badge)
![License](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)

</div>

---

## 📝 Descripción del Proyecto

**LINGUA-ANALYTICA V2.1** es una plataforma de evaluación comparativa que analiza y contrasta en tiempo real el rendimiento, precisión y estructura de dos motores de procesamiento de lenguaje natural (NLP) usados en entornos académicos:

* **Stanford Parser / Stanford CoreNLP** (Enfoque local/pesado)
* **Linguakit** empaquetado como servicio HTTP local

El sistema recibe una oración desde una interfaz web, ejecuta el análisis sintáctico en ambos motores, normaliza las estructuras de datos devueltas y genera de forma dinámica una **matriz de decisión automática** que determina cuál herramienta es óptima según las restricciones de los datos de entrada.

La ejecucion normal del sistema es **offline/local**: una vez construidas o descargadas las imagenes de contenedor, la aplicacion no depende de RapidAPI ni de servicios NLP externos en tiempo de ejecucion. La comunicacion ocurre solamente entre el navegador, el backend Rust y los servicios locales de Stanford CoreNLP y Linguakit.

> **Eje Temático de Investigación:** *Herramientas de Análisis Sintáctico NLP: creación de una matriz de decisión técnica justificando el uso de Stanford Parser o Linguakit según la naturaleza de los datos.*

---

## 🎯 Criterios de Evaluación y Objetivos

El motor de decisión técnica evalúa las respuestas basándose en los siguientes vectores métricos:

| Dimensión Analítica | Stanford CoreNLP | Linguakit |
| :--- | :--- | :--- |
| **Entorno de Ejecución** | Servicio HTTP local (`9000` interno, `19000` en host) | Servicio HTTP local (`3002`) |
| **Dependencia de Red** | Offline; solo red local entre contenedores | Offline; solo red local entre contenedores |
| **Arquitectura** | Gramáticas probabilísticas / Redes Neuronales | Reglas gramaticales + Estadístico |
| **Soporte de Idiomas** | Español mediante modelo `StanfordCoreNLP-spanish.properties` | Español por defecto (`LINGUAKIT_LANG=es`) |
| **Salida Estructural** | Árboles sintácticos complejos / Co-referencias | Dependencias planas y etiquetado POS formal |

### Objetivos clave de la matriz:
* Evaluar la robustez ante entradas ambiguas o con errores morfosintácticos.
* Garantizar la reproducibilidad del entorno de pruebas.
* Medir la utilidad de la salida estructurada para fases posteriores del compilador (AST / Análisis Semántico).

---

## 🛠️ Arquitectura del Sistema

El proyecto está organizado como un orquestador web en **Rust + Actix Web** y dos servicios NLP aislados en contenedores. La interfaz usa **HTMX** para enviar el formulario a `/analizar` y reemplazar el fragmento de resultados sin una SPA pesada. El backend renderiza plantillas con **Tera**, consulta ambos motores NLP, adapta sus respuestas a modelos internos comunes y entrega la matriz de decisión.

```text
          [ Usuario / Navegador ]
                    |
                    | HTMX POST /analizar
                    v
      [ Actix Web + Tera / app:8080 ]
                    |
          +---------+---------+
          |                   |
          v                   v
 [ Stanford Client ]  [ Linguakit Client ]
          |                   |
          | HTTP POST         | HTTP POST
          v                   v
 [ Stanford CoreNLP ] [ Linguakit API ]
 [ stanford:9000   ] [ linguakit:3002 ]
          |                   |
          +---------+---------+
                    |
                    v
        [ Normalizacion a modelos Rust ]
                    |
                    v
        [ Motor de decision tecnica ]
                    |
                    v
      [ templates/partials/resultado.html ]

```

### Componentes principales

| Componente | Archivo / servicio | Responsabilidad |
| :--- | :--- | :--- |
| Entrada web | `templates/index.html` | Formulario HTMX para capturar texto y tipo de análisis. |
| Rutas HTTP | `src/routes.rs` | Expone `/`, `/analizar`, `/api/stanford/local` y `/api/linguakit/local`. |
| Orquestador | `src/main.rs` | Inicializa Actix Web, carga variables `.env` y registra plantillas Tera. |
| Cliente Stanford | `src/stanford_client.rs` | Envía texto a Stanford CoreNLP y extrae tokens, lemas, POS y dependencias. |
| Cliente Linguakit | `src/linguakit_client.rs` | Consulta la API local de Linguakit; si no está disponible, intenta un binario local como fallback. |
| Modelo común | `src/models.rs` | Define estructuras serializables para tokens, dependencias, resultados y decisión. |
| Decisión técnica | `src/decision_engine.rs` | Puntúa ambas herramientas según salida disponible, estado y tipo de texto. |
| Resultado parcial | `templates/partials/resultado.html` | Renderiza comparación, dependencias y recomendación final. |

### Topología de contenedores

| Servicio | Imagen | Puerto interno | Puerto host | Compose |
| :--- | :--- | :--- | :--- | :--- |
| `app` | `ghcr.io/raul2811/lingua-analytica` | `8080` | `8080` | `compose.yaml`, `compose.prod.yaml` |
| `stanford` | `ghcr.io/raul2811/lingua-analytica-stanford` | `9000` | `19000` | `compose.yaml`, `compose.prod.yaml`, `compose.tools.yaml` |
| `linguakit` | `ghcr.io/raul2811/lingua-analytica-linguakit` | `3002` | `3002` | `compose.yaml`, `compose.prod.yaml`, `compose.tools.yaml` |

Los contenedores se ejecutan con `read_only`, `tmpfs` para `/tmp`, `no-new-privileges` y `cap_drop: ALL`. En `compose.prod.yaml` tambien usan `restart: unless-stopped`.

---

## 💻 Stack Tecnológico

* **Backend:** [Rust](https://www.rust-lang.org/) con [Actix-Web](https://actix.rs/) (Garantía de concurrencia segura y bajo tiempo de respuesta).
* **Frontend:** HTML5 con estilos CSS inline en plantillas Tera.
* **Interactividad:** [HTMX](https://htmx.org/) (Inyecciones de fragmentos HTML asíncronos para simular una SPA).
* **Integraciones:** `reqwest` (Rust) para consumir los servicios locales de Stanford CoreNLP y Linguakit.
* **Plantillas:** [Tera](https://tera.netlify.app/) para renderizar la vista principal y el parcial de resultados.
* **Contenedores:** Podman / podman-compose con imagenes publicables en GHCR.

---

## 🔗 Herramientas externas utilizadas

Este repositorio contiene una **implementacion propia** de la aplicacion web, el orquestador Rust, los clientes de integracion y la matriz de decision tecnica. Los contenedores de Stanford CoreNLP y Linguakit son una **contenerizacion del trabajo oficial de sus respectivos autores**, preparada para ejecutar ambas herramientas de forma reproducible y offline dentro del proyecto.

Las herramientas externas no se versionan dentro de `tools/`, no se suben archivos `.jar` al repositorio y no se incluyen claves ni API keys. Las imagenes se construyen desde fuentes oficiales durante el proceso de build; despues de ese paso, el analisis se ejecuta localmente sin llamadas a servicios externos.

### Linguakit

Linguakit se utilizo como herramienta NLP desarrollada por ProLNat@GE, CiTIUS, University of Santiago de Compostela. Inicialmente se evaluo el consumo via RapidAPI, pero se opto por la version open source local para evitar dependencia de endpoints externos, permitir ejecucion offline y asegurar reproducibilidad.

- Repositorio oficial: https://github.com/citiususc/Linguakit
- Contenerizacion local: `containers/linguakit/Containerfile`
- Modo de ejecucion usado como referencia:

```bash
./linguakit dep es /tmp/input.txt -a
```

En este proyecto Linguakit se emplea para obtener tokens, lemas, categorias gramaticales y dependencias sintacticas en espanol. El contenedor expone una API HTTP local en el puerto `3002`, consumida por el backend Rust mediante `LINGUAKIT_URL`.

### Stanford CoreNLP

Stanford CoreNLP se utilizo como suite Java de Stanford NLP para tokenizacion, segmentacion de oraciones, analisis sintactico, dependencias y otras tareas NLP. En esta implementacion se ejecuta como servidor HTTP local con modelos en espanol.

- Repositorio oficial: https://github.com/stanfordnlp/CoreNLP
- Pagina oficial de descarga: https://stanfordnlp.github.io/CoreNLP/download.html
- Modelo utilizado: `stanford-corenlp-4.5.10-models-spanish.jar`
- Contenerizacion local: `containers/stanford/Containerfile`

Modo de ejecucion usado como referencia:

```bash
java -mx4g -cp "*" edu.stanford.nlp.pipeline.StanfordCoreNLPServer \
  -serverProperties StanfordCoreNLP-spanish.properties \
  -port 9000 \
  -timeout 15000
```

Stanford CoreNLP se emplea para obtener tokens, lemas, etiquetas POS y dependencias gramaticales en formato JSON. El contenedor expone el puerto interno `9000`; en desarrollo local se publica como `19000` para evitar conflictos con servidores Stanford instalados en el host.

### Nota de licencias

El codigo propio de Lingua Analytica se publica bajo licencia MIT. Stanford CoreNLP y Linguakit mantienen sus licencias y autoria originales; este proyecto solo documenta y automatiza su ejecucion en contenedores para fines academicos y de comparacion tecnica.

---

## ⚙️ Requisitos Previos

Para desarrollar localmente:

1. **Rust Toolchain**
2. **Podman** y **podman-compose**

No necesitas instalar Java, Stanford CoreNLP ni Linguakit en el host. Esas herramientas se consumen desde imagenes.

---

## 🛠️ Instalación y Despliegue

### 1. Configuración del Entorno (`.env`)

Crea un archivo `.env` en la raíz del proyecto:

```bash
cp .env.example .env
```

Variables principales para desarrollo local con Rust en el host:

```env
APP_HOST=127.0.0.1
APP_PORT=8080
STANFORD_URL=http://localhost:19000
LINGUAKIT_URL=http://localhost:3002
LINGUAKIT_MODE=dep
LINGUAKIT_OUTPUT=-a
```

### 2. Desarrollo local

Levanta solo las herramientas NLP:

```bash
podman-compose -f compose.tools.yaml up -d
```

Ejecuta Rust en el host:

```bash
cargo run
```

Accede a `http://localhost:8080`.

Endpoints de verificación:

```bash
curl -sS "http://localhost:8080/api/stanford/local?texto=Hola%20mundo."
curl -sS "http://localhost:8080/api/linguakit/local?texto=Hola%20mundo."
```

### 3. Stack completo local

Ejecuta la aplicación y las herramientas desde imagenes:

```bash
podman-compose up -d
```

### 4. Despliegue completo con GHCR

```bash
GHCR_IMAGE_PREFIX=ghcr.io/raul2811 IMAGE_TAG=main podman-compose -f compose.prod.yaml up -d
```

### 5. Documentacion

Consulta:

- [Desarrollo local](documentacion/desarrollo-local.md)
- [Despliegue GHCR](documentacion/despliegue-ghcr.md)
- [Imagenes y contenedores](documentacion/imagenes-contenedores.md)

---

## 📄 Licencia

Este proyecto está bajo la Licencia MIT. Consulta el archivo `LICENSE` para más detalles.

---
