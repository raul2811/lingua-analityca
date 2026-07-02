# 📊 LINGUA-ANALYTICA V2.1
### Herramientas de Análisis Sintáctico NLP & Matriz de Decisión Técnica

Prototipo académico desarrollado para el curso **Diseño de Compiladores (INF511)** en la **Universidad de Panamá**.

---

## 🚀 badges

<div align="center">

![Rust](https://img.shields.io/badge/Rust-Actix_Web-orange?style=for-the-badge&logo=rust)
![HTMX](https://img.shields.io/badge/HTMX-Frontend-blue?style=for-the-badge&logo=htmx)
![TailwindCSS](https://img.shields.io/badge/TailwindCSS-UI-38B2AC?style=for-the-badge&logo=tailwindcss)
![NLP](https://img.shields.io/badge/NLP-Syntactic_Analysis-purple?style=for-the-badge)
![License](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)

</div>

---

## 📝 Descripción del Proyecto

**LINGUA-ANALYTICA V2.1** es una plataforma de evaluación comparativa que analiza y contrasta en tiempo real el rendimiento, precisión y estructura de dos de los motores de procesamiento de lenguaje natural (NLP) más utilizados en el ámbito académico e industrial:

* **Stanford Parser / Stanford CoreNLP** (Enfoque local/pesado)
* **Linguakit vía RapidAPI** (Enfoque en la nube/híbrido)

El sistema recibe un flujo de texto, ejecuta el análisis sintáctico en ambos motores de manera simultánea, normaliza las estructuras de datos devueltas y genera de forma dinámica una **matriz de decisión automática** que determina cuál herramienta es óptima según las restricciones de los datos de entrada.

> **Eje Temático de Investigación:** *Herramientas de Análisis Sintáctico NLP: creación de una matriz de decisión técnica justificando el uso de Stanford Parser o Linguakit según la naturaleza de los datos.*

---

## 🎯 Criterios de Evaluación y Objetivos

El motor de decisión técnica evalúa las respuestas basándose en los siguientes vectores métricos:

| Dimensión Analítica | Stanford CoreNLP | Linguakit |
| :--- | :--- | :--- |
| **Entorno de Ejecución** | Local (`localhost:9000`) | Cloud (RapidAPI) |
| **Dependencia de Red** | Nula (Offline) | Crítica (Requiere Internet) |
| **Arquitectura** | Gramáticas probabilísticas / Redes Neuronales | Reglas gramaticales + Estadístico |
| **Soporte de Idiomas** | Multilingüe (Configuración manual) | Nativo optimizado (Gallego, Español, Portugués, Inglés) |
| **Salida Estructural** | Árboles sintácticos complejos / Co-referencias | Dependencias planas y etiquetado POS formal |

### Objetivos clave de la matriz:
* Evaluar la robustez ante entradas ambiguas o con errores morfosintácticos.
* Garantizar la reproducibilidad del entorno de pruebas.
* Medir la utilidad de la salida estructurada para fases posteriores del compilador (AST / Análisis Semántico).

---

## 🛠️ Arquitectura del Sistema

El sistema utiliza una arquitectura desacoplada impulsada por **HTMX** para actualizaciones reactivas sin JavaScript pesado, procesado en el backend por la velocidad y seguridad de memoria de **Rust**.

```text
       [  Usuario / Interfaz Web  ]
                   │
                   ▼ (HTMX / Hypermedia Exchanges)
       [ Backend Actix Web (Rust) ]
                   │
         ┌─────────┴─────────┐
         ▼                   ▼
┌─────────────────┐ ┌─────────────────┐
│ Cliente Local   │ │ Cliente Cloud   │
│ Stanford CoreNLP│ │ Linguakit API   │
│ (Port 9000)     │ │ (RapidAPI Gateway)
└─────────────────┘ └─────────────────┘
         │                   │
         └─────────┬─────────┘
                   ▼
       [ Normalizador de Datos ]
                   │
                   ▼
       [ Motor de Decisión Técnica ] ──► Generación de Matriz

```

---

## 💻 Stack Tecnológico

* **Backend:** [Rust](https://www.rust-lang.org/) con [Actix-Web](https://actix.rs/) (Garantía de concurrencia segura y bajo tiempo de respuesta).
* **Frontend:** HTML5 + [TailwindCSS](https://tailwindcss.com/) (Diseño limpio, responsivo y profesional).
* **Interactividad:** [HTMX](https://htmx.org/) (Inyecciones de fragmentos HTML asíncronos para simular una SPA).
* **Integraciones:** `reqwest` (Rust) para consumo de API local de Stanford y endpoints remotos de Linguakit.

---

## ⚙️ Requisitos Previos

Para desarrollar localmente:

1. **Rust Toolchain**
2. **Podman** y **podman-compose**

No necesitas instalar Java, Stanford CoreNLP ni Linguakit en el host. Esas herramientas se consumen desde imagenes.

---

## 🛠️ Instalación y Despliegue

### 1. Configuración del Entorno (.env)

Crea un archivo `.env` en la raíz del proyecto Rust:

```bash
cp .env.example .env
```

### 2. Desarrollo local

Levanta solo Stanford y Linguakit:

```bash
podman-compose -f compose.tools.yaml up -d
```

Ejecuta Rust en el host:

```bash
cargo run
```

Accede a `http://localhost:8080`.

### 3. Despliegue completo con GHCR

```bash
GHCR_IMAGE_PREFIX=ghcr.io/raul2811 IMAGE_TAG=main podman-compose -f compose.prod.yaml up -d
```

### 4. Documentacion

Consulta:

- [Desarrollo local](documentacion/desarrollo-local.md)
- [Despliegue GHCR](documentacion/despliegue-ghcr.md)
- [Imagenes y contenedores](documentacion/imagenes-contenedores.md)

---

## 📄 Licencia

Este proyecto está bajo la Licencia MIT. Consulta el archivo `LICENSE` para más detalles.

---
