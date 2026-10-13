# Plantillas Typst

Cada plantilla genera un documento. Se incrustan en el ejecutable al compilar (`crates/mantenix-informes`).

| Plantilla | Documento | Entradas (`sys.inputs`, texto JSON) |
|---|---|---|
| `hoja_costes.typ` | Hoja de costes interna | `datos` (Valoracion), `empresa` |
| `tarifa.typ` | Tarifa propia | `datos` (Tarifa), `empresa` |
| `contrato.typ` | Contrato de mantenimiento | `datos` (contrato), `empresa`, `operaciones` |
| `comun.typ` | Estilo común y formato de números | — |

Para cambiar el aspecto de un documento basta con editar su plantilla y recompilar. Para probar una plantilla sin la app, use la línea de comandos (`mantenix valorar … --pdf`, `mantenix contrato … --pdf`).
