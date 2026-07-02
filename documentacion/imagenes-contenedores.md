# Imagenes y decisiones de contenedorizacion

## `containers/app/Containerfile`

Empaqueta la aplicacion Rust con build multi-stage. La etapa final solo contiene el binario, plantillas, estaticos y certificados CA.

Motivo: reducir tamano de runtime y evitar incluir toolchain Rust en produccion.

## `containers/stanford/Containerfile`

Descarga Stanford CoreNLP 4.5.10 y el modelo espanol durante el build. El servidor arranca con:

```text
StanfordCoreNLP-spanish.properties
```

Motivo: no versionar JARs pesados en Git y mantener Stanford configurado para espanol.

## `containers/linguakit/Containerfile`

Clona Linguakit desde GitHub durante el build e instala dependencias Perl necesarias para su API.

Motivo: evitar guardar el repositorio Linguakit dentro de este repo y publicar una API HTTP consumible desde Rust.

## Seguridad aplicada

Los compose usan:

```text
read_only: true
tmpfs: /tmp
no-new-privileges
cap_drop: ALL
usuarios no root en las imagenes
```

Motivo: reducir permisos de runtime y seguir buenas practicas basicas de contenedores.
