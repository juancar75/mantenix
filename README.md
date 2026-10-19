# Mantenix

Aplicación de escritorio, de código abierto, para **valorar contratos de mantenimiento** de instalaciones térmicas (RITE) y de protección contra incendios (RIPCI) en edificios no industriales, y generar la **hoja de costes, la tarifa y el contrato** en PDF. También hace **cartas presupuesto** de reparaciones y pequeñas obras.

> Estado: **versión 0.1 en desarrollo**. El motor de cálculo y la aplicación funcionan; los tiempos estándar son un borrador pendiente de calibrar con datos reales. Ver [CHANGELOG](CHANGELOG.md).

## Qué hace

1. Se registra el **inventario** de un edificio: equipos RITE y PCI, potencia, uso, desplazamiento.
2. Mantenix aplica la **normativa vigente** (tablas 3.1 y 3.3 del RITE, anexo II del RIPCI tras el RD 164/2025) y calcula las visitas mínimas y las operaciones de cada equipo.
3. Calcula **horas, coste y precio de venta** con los parámetros del año: coste hora, desplazamiento, subcontratas, indirectos y margen.
4. Genera la **hoja de costes interna**, la **tarifa propia** y el **contrato** en PDF.
5. Hace **cartas presupuesto** de reparaciones y pequeñas obras: qué se ha roto o desgastado, el repuesto propuesto y el importe con IVA, con el margen interno a la vista solo en la app.
6. Guarda **todo en SQLite**, con un **historial de cambios** de cada dato (quién, cuándo, valor anterior).

## Instalación (usuarios)

Descargue el instalador de la última versión en [Releases](../../releases) (Windows `.msi`/`.exe`, Linux `.deb`/`.AppImage`, macOS `.dmg`).
La primera vez se crea una base con **datos de ejemplo**. Para trabajar con los datos de su empresa: *Base de datos → Crear base con datos de empresa…* (ver [docs/datos-privados.md](docs/datos-privados.md)).

## Desarrollo

Requisitos: [Rust](https://rustup.rs) estable, [Node.js](https://nodejs.org) 20 o superior y las [dependencias de Tauri](https://v2.tauri.app/start/prerequisites/) de su sistema.

```bash
cargo test --workspace --exclude mantenix-app   # pruebas del núcleo
cd app && npm install && npm run tauri dev       # aplicación de escritorio en modo desarrollo
cd app && npm run tauri build                    # instalador
```

Línea de comandos (útil para pruebas y automatización):

```bash
cargo run -p mantenix-cli -- init
cargo run -p mantenix-cli -- importar datos/semilla/ejemplo/contratos/oficinas-vrf.json
cargo run -p mantenix-cli -- valorar "Ejemplo - Oficinas con VRF" --pdf salida/hoja.pdf
cargo run -p mantenix-cli -- historial
```

## Estructura

```
crates/
  mantenix-core/      Motor: SQLite, normativa, régimen, horas, costes, precio, tarifa. Sin interfaz.
  mantenix-informes/  PDF con Typst (plantillas y fuentes incrustadas en el ejecutable).
  mantenix-cli/       Línea de comandos.
app/
  src/                Interfaz (Vue 3 + TypeScript).
  src-tauri/          Aplicación de escritorio (Tauri 2): expone el núcleo a la interfaz.
plantillas/           Plantillas Typst de los documentos.
datos/semilla/
  normativa/          Operaciones RITE y RIPCI transcritas del BOE, con su fuente y estado de cotejo.
  ejemplo/            Datos de empresa FICTICIOS para probar.
docs/                 Arquitectura, método de cálculo, normativa, trazabilidad, decisiones.
```

## Documentación

- [Arquitectura](docs/arquitectura.md)
- [Método de cálculo](docs/metodo-calculo.md)
- [Normativa: fuentes y estado de cotejo](docs/normativa.md)
- [Trazabilidad](docs/trazabilidad.md)
- [Datos privados de la empresa](docs/datos-privados.md)
- [Registro de decisiones](docs/decisiones/)
- [Cómo colaborar](CONTRIBUTING.md)

## Aviso

Mantenix es una herramienta de apoyo. Los precios y las operaciones que propone deben revisarse por un técnico competente antes de ofertar o firmar un contrato. La normativa transcrita no sustituye al texto oficial publicado en el BOE.

## Licencia

Código bajo [licencia MIT](LICENSE). La tipografía Inter se distribuye bajo [SIL OFL 1.1](assets/fuentes/OFL.txt). Los textos normativos proceden del BOE y no están sujetos a derechos de autor (art. 13 de la Ley de Propiedad Intelectual).
