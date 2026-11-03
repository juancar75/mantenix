# Registro de cambios

Formato basado en [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/). Versiones según [SemVer](https://semver.org/lang/es/).

## [Sin publicar]

### Añadido
- **Catálogo ampliado** (13 equipos, con tiempos estándar, operaciones y tasa de avisos):
  - Ventilación: extractor de garaje que no es de humos (`VENT-GAR`), extractor de cuartos técnicos, trasteros o basuras (`VENT-CT`) y cuadro de maniobra de ventilación (`VENT-CUADRO`).
  - Control de humos: cuadro de mando de bomberos (`SCH-BOMB`) y kit de sobrepresión de escalera (`SOBREP-KIT`), con las operaciones del RIPCI para control de humos y, en el kit, la medición anual de presión diferencial y velocidad en puerta (UNE-EN 12101-6).
  - Climatización: bomba de calor geotérmica o agua-agua, deshumectadora, humidificador, cortina de aire, depósito de inercia, intercambiador de placas, emisores (radiadores) y suelo radiante por colector.
- **Tabla 7.1 del CTE DB HS 3** (`datos/semilla/normativa/cte_hs3_tabla_7_1_operaciones.csv`): limpieza anual de conductos, aberturas y extractores, revisión semestral de filtros y automatismos, y comprobaciones cada 5 años, que se listan fuera de la cuota. Pendiente de cotejo con el texto vigente.
- El asistente pregunta por todos ellos (también por el ventilador de climatización `VENT`, que faltaba) y el contrato tipo los incluye: bloque «Ventilación del garaje y de cuartos técnicos» con sus operaciones agrupadas por periodicidad, y notas de UNE-EN 12101-6 y del cuadro de bomberos en control de humos.
- Las bases ya creadas reciben los equipos y operaciones nuevos al abrirse (o con `mantenix init`), sin tocar los tiempos ni los equipos editados.

- **Limpiezas aparte**: las limpiezas que exige la normativa (evaporadores y condensadores, torres, circuito y conductos de humos y chimeneas, biomasa, y las del DB HS 3) no entran en la cuota. Siguen en el programa del contrato marcadas *(se presupuesta aparte)*, con precio orientativo por intervención y al año (mismos costes, indirectos y margen). Se puede cambiar por equipo y operación en «Catálogo y tiempos» (columna *Cobro*) o con `mantenix cobro`.
- **Inspecciones reglamentarias** (`datos/semilla/normativa/inspecciones_periodicas.csv`): RITE (eficiencia energética cada 4 años e instalación completa cada 15), RIPCI art. 22 (cada 10 años), REBT ITC-BT-05 (riesgo cada 5 años, zonas comunes > 100 kW cada 10), gas canalizado ITC-ICG 07 (5 años), depósitos de GLP (revisión 5 años, prueba 15 años) y gasóleo ITC MI-IP03 (revisión 5 o 10 años, inspección OCA 10 años). El asistente sugiere las que corresponden por el inventario; el contrato las lista con próxima fecha, acompañamiento (horas `horas_inspeccion`, 3 por defecto, a la hora de oficial) y precio del organismo, todo aparte. La valoración avisa de las que faltan.

### Cambiado
- **Instalaciones de 70 kW o menos**: todas las operaciones se hacen con la frecuencia de revisión de la tabla 3.1 del RITE (anual o cada 2-4 años según equipo y uso). Las periodicidades de la tabla 3.3 se aplican solo por encima de 70 kW. Revisadas todas contra la tabla 3.3: coinciden.
- DB HS 3: la revisión de automatismos de los sistemas de control es cada 2 años (no semestral).

- **RITE cotejado con el BOE** (RD 1027/2007 consolidado): tablas 3.1 y 3.3 coinciden (corregidos los textos de las operaciones 39 y 42). Inspecciones de eficiencia energética **cada 4 años** (no 5) para generadores de calor o frío de más de 70 kW, e inspección de la instalación completa **cada 15 años** cuando tiene más de 15 años (no 10); exentas las de contrato de rendimiento energético o con automatización según la IT 1.2.4.3.5. La biomasa de 70 kW o menos sigue las periodicidades de la tabla 3.3, como dice la IT 3.3.
- **RIPCI cotejado** con la Guía técnica de aplicación v4 (junio 2025, texto del RD 513/2017 con el RD 164/2025): las 97 operaciones del anexo II (tablas I, II y III) coinciden en texto y periodicidad. El art. 22 queda verificado: exentos el residencial vivienda y los usos pequeños, pero el aparcamiento de 500 m² o más se inspecciona aunque el edificio sea de viviendas; con solo extintores o alumbrado de emergencia no hay inspección.
- Control de humos: el mantenimiento sigue además la UNE 23584 en edificios de una planta, con atrios, con escaleras y subterráneos (no solo subterráneos); la sobrepresión de escaleras, UNE-EN 12101-6 y UNE 23584.

### Corregido
- Textos de operaciones de fabricante que se cortaban por contener un punto y coma (sustitución de filtros de fancoil).

## [0.2.0] - 2026-11-03

### Añadido
- **Asistente de nuevo contrato** en la app: 8 pasos guiados (cliente, edificio, climatización, incendios, garaje y emergencias, avisos y condiciones, resultado) con ayuda en cada pantalla; al final, precio, hoja de costes y contrato en PDF. Guía en `docs/guia-nuevo-contrato.md`.
- Los datos del documento (cliente, edificio, firmante, referencia) se guardan con el contrato; botón *Contrato PDF* también en «Contratos».
- **Contrato tipo** generado desde el inventario y la valoración (`mantenix contrato-pdf <contrato> --partes <json> --pdf`): instalaciones térmicas (operaciones del RITE con su periodicidad), protección contra incendios (anexo II del RIPCI), ventilación por CO y alumbrado de emergencia; importe en cifras y letras, pago según la periodicidad de las visitas y cláusula de avisos.
- **Avisos de avería** en la valoración: estimación por equipo, antigüedad y uso; modalidades bolsa (por defecto), incluidos y aparte. Ejemplo `datos/semilla/ejemplo/contratos/edificio-tipo.json`.
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
- Las bases de versiones anteriores se actualizan solas al abrirse (columnas nuevas y disparadores de auditoría).
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
