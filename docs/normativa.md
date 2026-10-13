# Normativa: fuentes y estado de cotejo

Cada operación de `datos/semilla/normativa/` tiene las columnas `fuente` y `verificado`. La aplicación las muestra en *Catálogo y tiempos*.

| Fichero | Contenido | Fuente | Estado |
|---|---|---|---|
| `ripci_anexo_II_operaciones.csv` | RIPCI (RD 513/2017), anexo II, tablas I, II y III: 97 operaciones | Redacción tras el RD 164/2025, según la *Guía técnica de aplicación del RIPCI*, v4 (junio 2025), Ministerio de Industria | Texto cotejado palabra a palabra con la guía (`si_guia_v4`). **Pendiente**: cotejo final con el BOE del RD 164/2025 |
| `rite_tabla_3_3_operaciones.csv` | RITE (RD 1027/2007), IT 3.3, tabla 3.3: 43 operaciones (>70 kW) | Transcripción de un pliego público que reproduce el RITE consolidado (BOE 02/08/2022) | **Sin cotejar** con el BOE. Filas 35, 39 y 42 completadas |
| `rite_tabla_3_1_frecuencia_revisiones.csv` | RITE, IT 3.3, tabla 3.1: frecuencia de revisión ≤70 kW | Ídem | **Sin cotejar**. Fila `RITE-T31-04` dudosa (≤70 kW según coherencia de la tabla) |
| `operaciones_fabricante.csv` | Operaciones sin tabla reglamentaria (detección de CO…) | Instrucciones de fabricante, CTE DB HS 3 | Criterio técnico |

## Pendiente (issues abiertos)

1. Cotejar las tablas 3.1 y 3.3 del RITE con el PDF consolidado del BOE (BOE-A-2007-15820).
2. Cotejar el anexo II del RIPCI con el BOE del RD 164/2025.
3. Añadir la legionela (RD 487/2022) y el control de fugas de gases fluorados (Reglamento UE 2024/573) como operaciones opcionales.

## Normas UNE

El RIPCI remite a normas UNE (23120, UNE-EN 671-3, 23007-14, UNE-EN 12845…). **Su texto está protegido por derechos de autor y no se incluye.** Solo se cita la referencia. Cada empresa debe disponer de las normas por su cuenta.
