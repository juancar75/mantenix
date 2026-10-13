# 0001. Rust, SQLite y Typst

- Estado: aceptada
- Fecha: 2026-10-13

## Contexto
La aplicación debe funcionar sin conexión, ser fiable en los cálculos, generar PDF profesionales y ser mantenible por un equipo pequeño durante años.

## Decisión
- **Rust** para todo el núcleo: tipado estricto, pruebas integradas, un solo ejecutable sin dependencias.
- **SQLite** (`rusqlite`, compilado dentro del programa) como única base de datos: un fichero, sin servidor, copiable.
- **Typst** como motor de PDF, usado como biblioteca: plantillas legibles, editables sin programar, y compilación en milisegundos.

## Consecuencias
- Los PDF se generan igual en todos los equipos (plantillas y fuentes incrustadas).
- Quien solo quiera cambiar el aspecto de un documento edita un `.typ`.
- No se usan Excel ni Word como formato de trabajo.
