# 0003. Separar datos públicos y privados

- Estado: aceptada
- Fecha: 2026-10-13

## Contexto
El repositorio es público y colaborativo, pero la empresa trabaja con tarifas de proveedores, costes y contratos confidenciales.

## Decisión
- En el repositorio: normativa del BOE (dominio público) y datos de empresa **ficticios** de ejemplo.
- Datos reales: carpeta `privado/` (ignorada por Git) y bases `*.db` (ignoradas).
- Texto de normas UNE: nunca; solo referencias.

## Consecuencias
Cada empresa crea su base a partir de su carpeta privada. Ver `docs/datos-privados.md`.
