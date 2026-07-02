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

Para desplegar el prototipo localmente, asegúrate de contar con:

1. **Rust Toolchain** (v1.75+ recomendado)
2. **Java JDK 8 o superior** (Requerido para ejecutar el servidor de Stanford CoreNLP)
3. **API Key de Linguakit** a través de la plataforma [RapidAPI](https://rapidapi.com/).

---

## 🛠️ Instalación y Despliegue

### 1. Servidor Stanford CoreNLP

Descarga el parser oficial e inicia el servidor en el puerto por defecto:

```bash
java -mx4g -cp "stanford-corenlp-4.x.x/*" edu.stanford.nlp.pipeline.StanfordCoreNLPServer -port 9000 -timeout 15000

```

### 2. Configuración del Entorno (.env)

Crea un archivo `.env` en la raíz del proyecto Rust:

```env
STANFORD_URL=http://localhost:9000
LINGUAKIT_API_KEY=tu_api_key_aqui
PORT=8080

```

### 3. Compilación y Ejecución del Proyecto

```bash
# Clonar el repositorio
git clone [https://github.com/tu-usuario/lingua-analytica.git](https://github.com/tu-usuario/lingua-analytica.git)
cd lingua-analytica

# Compilar en modo desarrollo y ejecutar
cargo run

```

Accede a la aplicación navegando a `http://localhost:8080`.

---

## 📄 Licencia

Este proyecto está bajo la Licencia MIT. Consulta el archivo `LICENSE` para más detalles.

---

---

### Cambios clave aplicados:

* **Estructura Limpia:** Uso de separadores visuales (`---`) y jerarquía lógica de encabezados (`##`, `###`).
* **Tabla Comparativa:** Añadida una tabla de dimensiones analíticas para vender la idea del proyecto de inmediato al profesor o evaluador.
* **Cajas de Citas (`>`)**: Resaltan el tema principal asignado en el curso.
* **Diagrama de Flujo:** Rediseñado con cajas de texto alineadas (`┌──┴──┐`) para que se lea perfectamente en el renderizador de Markdown de GitHub/GitLab.
* **Secciones de Despliegue Claras:** Pasos ordenados con bloques de código listos para producción académica.
