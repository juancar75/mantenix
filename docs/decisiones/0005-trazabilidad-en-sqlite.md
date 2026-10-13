# 0005. Registro de cambios con disparadores SQLite

- Estado: aceptada
- Fecha: 2026-10-13

## Contexto
Se exige trazabilidad: saber quién cambió un tiempo, un parámetro o un inventario, y su valor anterior.

## Decisión
Disparadores `AFTER INSERT/UPDATE/DELETE` en las tablas de datos escriben en `auditoria` (JSON antes/después) con el usuario de la tabla `sesion`. Cada valoración se guarda completa.

## Consecuencias
- Ninguna parte de la aplicación puede saltarse el registro.
- La tabla crece con el uso; se podrá archivar por años si hiciera falta.
