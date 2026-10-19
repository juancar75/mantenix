# Registro de cambios

Formato basado en [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/). Versiones según [SemVer](https://semver.org/lang/es/).

## [Sin publicar]

### Añadido
- **Contrato tipo** generado desde el inventario y la valoración (`mantenix contrato-pdf <contrato> --partes <json> --pdf`): instalaciones térmicas (operaciones del RITE con su periodicidad), protección contra incendios (anexo II del RIPCI), ventilación por CO y alumbrado de emergencia; importe en cifras y letras, pago según la periodicidad de las visitas y cláusula de avisos.
- **Avisos de avería** en la valoración: estimación por equipo, antigüedad y uso; modalidades bolsa (por defecto), incluidos y aparte. Ejemplo `datos/semilla/ejemplo/contratos/edificio-tipo.json`.

## [0.2.0] - 2026-10-19

### Añadido
- **Presupuestos** de reparación (correctivo) y de pequeñas obras o mejoras, con numeración automática `P-AAAA-NNN`, estados (borrador, enviado, aceptado, rechazado) y registro de cambios.
- **Carta presupuesto** en PDF: explica qué se ha roto o desgastado, las consecuencias de no actuar y el repuesto o trabajo propuesto; importe desglosado con IVA, condiciones estándar y espacio para la aceptación del cliente. No muestra costes internos.
- Líneas de repuesto, hora de oficial, hora de oficial + ayudante, desplazamiento y otras partidas, con coste interno y margen del presupuesto.
- Parámetros por año para presupuestos: `precio_hora_oficial`, `precio_hora_pareja`, `precio_salida`, `coste_salida` y `recargo_material_pct` (el precio del material se calcula desde su coste).
- **Listado de presupuestos** por cliente e instalación, con filtros (cliente, ámbito, estado), resumen por estado y **relación en PDF**.
- **Ámbito** del presupuesto (RITE, PCI, baja tensión): la carta muestra el logo y el número de empresa habilitada de ese ámbito.
- Pantalla **Empresa y logos**: datos del membrete, habilitaciones y logo por ámbito (guardados en la base, con historial).
- **Modelos de presupuesto**: 28 averías típicas de semilla (sala de calderas, ACS, bomba de calor aire-agua, climatizador, extractor, split, PCI, alumbrado de emergencia, CO de garaje) en `datos/semilla/presupuestos/modelos.json`, con precios orientativos. Cualquier presupuesto se puede guardar como modelo propio para adaptarlo a equipos parecidos. Las bases existentes reciben los modelos al abrirse.
- Órdenes `mantenix presupuesto <json> --pdf`, `mantenix presupuestos [--cliente] [--pdf]` y `mantenix logo <ámbito> <fichero>`. Ejemplo en `datos/semilla/ejemplo/presupuestos/`.

### Cambiado
- La aplicación recuerda la última base de datos abierta y el último usuario identificado.

## [0.1.0] - 2026-10-13

### Añadido
- Operación de empresa `FAB-FILTRO-SUST`: sustitución trimestral de filtros de unidades terminales (la comprobación mensual sigue siendo RITE-T33-19). Tiempo para fancoils en el ejemplo.
- Orden `mantenix empresa` para cargar los datos de la empresa en la base.
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

### Cambiado
- El usuario de la auditoría se identifica en cada sesión; ya no se hereda de una sesión anterior.
- Tabla de inventario de la app con anchos fijos.
