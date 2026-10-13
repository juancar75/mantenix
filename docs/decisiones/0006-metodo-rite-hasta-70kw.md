# 0006. Frecuencia de operaciones en RITE ≤ 70 kW

- Estado: aceptada
- Fecha: 2026-10-13

## Contexto
La tabla 3.1 del RITE fija cada cuánto se revisan las instalaciones ≤70 kW, pero los contratos suelen incluir más visitas (p. ej. 4 al año). Hacer la revisión completa en cada visita sobrevaloraba el precio (calibración con un contrato real de oficinas con VRF: +75 %).

## Decisión
Las operaciones de temporada (`t`) siguen la frecuencia de la tabla 3.1; las más frecuentes (`m`, `2t`) se hacen en cada visita contratada.

## Estado
Provisional: validar con partes de trabajo reales (issue de calibración).
