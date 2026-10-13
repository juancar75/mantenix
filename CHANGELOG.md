# Registro de cambios

Formato basado en [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/). Versiones según [SemVer](https://semver.org/lang/es/).

## [Sin publicar]

## [0.1.0] - 2026-10-13

### Añadido
- Núcleo `mantenix-core`: régimen normativo RITE (≤70 / >70 kW) y RIPCI, horas, costes, precio y tarifa propia.
- Base de datos SQLite con semilla, contratos, valoraciones y registro de cambios (`auditoria`) por usuario.
- Normativa: RIPCI anexo II (97 operaciones, redacción RD 164/2025) y RITE IT 3.3 (tablas 3.1 y 3.3).
- Informes PDF con Typst: hoja de costes, tarifa propia y contrato; fuentes Inter incrustadas.
- Aplicación de escritorio (Tauri 2 + Vue 3): contratos, catálogo y tiempos, parámetros por año, tarifa, historial y gestión de la base.
- Línea de comandos `mantenix`.
- Documentación: arquitectura, método de cálculo, normativa, trazabilidad, datos privados, decisiones.

### Pendiente de verificar
- Tablas 3.1 y 3.3 del RITE sin cotejar con el BOE.
- Tiempos estándar en borrador.
