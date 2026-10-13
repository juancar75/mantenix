# Arquitectura

```
┌──────────────────────────── Aplicación de escritorio (Tauri 2) ────────────────────────────┐
│  app/src  (Vue 3 + TypeScript)          app/src-tauri  (Rust)                              │
│  Vistas: Contratos, Catálogo,   ──invoke──►  #[tauri::command]  ──►  mantenix-core          │
│  Parámetros, Tarifa, Historial,                                 ──►  mantenix-informes      │
│  Base de datos                                                                             │
└────────────────────────────────────────────────────────────────────────────────────────────┘
                                         │
mantenix-cli (línea de comandos) ────────┤  (mismas funciones)
                                         ▼
        mantenix-core ─────────► SQLite (mantenix.db)
          datos.rs     estructuras de la biblioteca (normativa + empresa)
          db.rs        esquema, semilla, auditoría, contratos, valoraciones
          modelo.rs    inventario de un contrato (entrada)
          motor.rs     régimen normativo → horas → costes → precio
          tarifa.rs    tarifa propia por equipo
        mantenix-informes ─────► Typst ─► PDF (plantillas y fuentes incrustadas)
```

## Principios

1. **El núcleo no conoce la interfaz.** Toda regla de cálculo está en `mantenix-core`, con pruebas. Así se puede cambiar la interfaz (o añadir una versión para tableta) sin tocar el cálculo.
2. **SQLite es la única fuente de datos.** Los CSV de `datos/semilla/` solo sirven para crear una base nueva.
3. **La normativa es un dato, no código.** Las operaciones, periodicidades y fuentes están en tablas; un cambio normativo no exige reprogramar.
4. **Nada se pierde.** Los cambios en los datos quedan en `auditoria`; cada valoración se guarda completa en `valoracion`.
5. **Los documentos son plantillas.** El aspecto de los PDF se cambia editando `plantillas/*.typ`, sin tocar Rust.

## Tablas SQLite

| Grupo | Tabla | Contenido |
|---|---|---|
| Normativa | `operacion` | Operaciones RITE (tabla 3.3), RIPCI (anexo II) y de fabricante, con periodicidad, fuente y estado de cotejo |
| | `tabla_3_1` | Frecuencia de revisión RITE ≤70 kW según equipo, potencia y uso |
| Empresa | `equipo` | Catálogo de tipos de equipo |
| | `tiempo` | Minutos y técnicos por equipo y operación |
| | `parametro` | Parámetros económicos por año |
| | `tarifa_subcontrata` | Precios de subcontratas por año |
| | `empresa` | Datos de la empresa mantenedora (JSON) |
| Contratos | `cliente`, `contrato`, `instalacion`, `linea_equipo` | Inventario (UUID + `actualizado`) |
| | `valoracion` | Resultado completo de cada valoración |
| Trazabilidad | `auditoria`, `sesion` | Registro de cambios y usuario activo |

## Por qué estas tecnologías

Ver [decisiones](decisiones/).
