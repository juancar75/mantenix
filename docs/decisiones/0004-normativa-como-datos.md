# 0004. La normativa es un dato, no código

- Estado: aceptada
- Fecha: 2026-10-13

## Contexto
El RITE y el RIPCI cambian (RD 178/2021, RD 164/2025). Si las operaciones estuvieran en el código, cada cambio exigiría programar.

## Decisión
Las operaciones, periodicidades y frecuencias están en tablas (`operacion`, `tabla_3_1`) con `fuente` y `verificado`. Las plantillas de contrato leen las operaciones de la base de datos.

## Consecuencias
- Un cambio normativo es un cambio de datos revisable en una PR.
- Los `id` de operación son estables: si una norma sustituye una operación, se añade otra fila.
