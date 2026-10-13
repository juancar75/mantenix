# Método de cálculo

Implementado en `crates/mantenix-core/src/motor.rs` (función `valorar`). Cada paso tiene pruebas en `lib.rs`.

## 1. Régimen normativo y visitas mínimas

| Instalación | Regla | Visitas mínimas |
|---|---|---|
| RITE ≤ 70 kW | Tabla 3.1 (IT 3.3): meses entre revisiones según equipo, potencia y uso (viviendas / otros) | Tantas como exija el equipo más exigente (mínimo 1) |
| RITE > 70 kW | Tabla 3.3: cada operación con su periodicidad (`m`, `t`, `2t`…) | La frecuencia más alta (p. ej. `m` → meses de temporada) |
| PCI (RIPCI) | Anexo II, tablas I y II (redacción RD 164/2025) | 4 (trimestral) |
| Otras (ventilación CO…) | Instrucciones del fabricante | 1 |

Si las visitas contratadas son menos que las mínimas, la valoración muestra un aviso.

## 2. Veces al año de cada operación

| Código | Significado | Veces/año |
|---|---|---|
| `m` | Mensual en temporada | meses de temporada (máximo: visitas contratadas) |
| `t` / `1a` | Una vez por temporada / año | 1 |
| `2t` / `6m` | Dos veces por temporada / semestral | 2 |
| `3m` | Trimestral | 4 |
| `S` | Semanal | 52 |
| `S*` | Semanal, la hace el usuario | 0 (no se presupuesta) |
| `5a`, `10a`, `20a`, `25a`, `vida_util` | Varios años | 0: se listan aparte («fuera de cuota») |

En RITE ≤ 70 kW, las operaciones de temporada (`t`) siguen la frecuencia de la tabla 3.1 (p. ej. cada 2 años → 0,5 veces/año). Las de mayor frecuencia se hacen en cada visita contratada.

Ninguna operación se cuenta más veces que visitas haya (excepto `S`).

## 3. Horas y mano de obra

```
horas = minutos/60 × cantidad × veces/año
coste = horas × (coste_hora_oficial + (técnicos − 1) × coste_hora_ayudante)
```

## 4. Desplazamiento y tiempo fijo (solo si hay trabajo con medios propios)

```
horas = visitas × (2 × minutos_ida + tiempo_fijo_visita) / 60
coste = horas × coste_hora_oficial + visitas × 2 × km_ida × coste_km
```

El coste fijo del vehículo está dentro del coste hora del oficial; `coste_km` solo cubre combustible y desgaste.

## 5. Subcontrata

```
por_visita = máx( Σ cantidad × precio_partida , mínimo_por_visita )
anual = por_visita × visitas_de_subcontrata
```

Partidas `incluido` → 0. Partidas `consultar` o `aparte`, o sin precio → aviso.

## 6. Precio

```
directo      = mano de obra + consumibles + desplazamiento + subcontrata
coste total  = directo × (1 + indirectos% + contingencia%)
precio       = coste total / (1 − margen%)        ← margen sobre venta
```

Si el contrato tiene precio actual, se muestra el **margen real** que deja: `1 − coste / precio_actual`.

## 7. Tarifa propia

Para cada equipo con tiempos propios:

- **Revisión completa**: todas sus operaciones una vez.
- **Anual reglamentaria**: periodicidades completas con 12 meses de uso.

Precio = coste × (1 + recargo) / (1 − margen). Los equipos subcontratados toman el precio de la partida.

## Parámetros por año

Si un año no tiene un parámetro, se usa el del año anterior más próximo. Así, en enero solo se dan de alta los valores que cambian.
