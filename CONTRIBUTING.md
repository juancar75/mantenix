# Cómo colaborar en Mantenix

Gracias por colaborar. Esta guía está pensada también para quien **no ha usado Git nunca**.

## Reglas básicas

1. **Nunca se suben datos reales de empresa ni de clientes** (tarifas de proveedores, costes, contratos, inventarios con nombres). Van en la carpeta `privado/`, que Git ignora. Ver [docs/datos-privados.md](docs/datos-privados.md).
2. **Nunca se sube texto de normas UNE**: solo su referencia. El texto del BOE sí se puede incluir.
3. **Todo cambio pasa por una *pull request*** (PR) revisada por otra persona. Nadie sube directamente a `main`.
4. **Toda regla de cálculo va en `mantenix-core`** y con su prueba automática. La interfaz solo muestra y recoge datos.
5. **Todo dato normativo lleva su fuente** (BOE, guía técnica…) y su estado de cotejo.

## Primeros pasos con GitHub Desktop (sin línea de comandos)

1. Instale [GitHub Desktop](https://desktop.github.com) e inicie sesión.
2. *File → Clone repository* y elija `mantenix`.
3. Antes de cambiar nada: **Current branch → New branch**, con un nombre que diga qué va a hacer, por ejemplo `tiempos-vrf` o `corregir-tabla-3-1`.
4. Haga los cambios. GitHub Desktop muestra a la izquierda los ficheros modificados.
5. Escriba un **resumen del cambio** (ver formato abajo) y pulse **Commit**.
6. Pulse **Publish branch** y después **Create Pull Request**. Rellene la plantilla que aparece.
7. Otra persona revisa, comenta y aprueba. Cuando está aprobada, se pulsa **Merge**.

## Formato de los mensajes de commit

Una línea corta en español, empezando por el tipo de cambio:

| Tipo | Uso | Ejemplo |
|---|---|---|
| `feat:` | Nueva función | `feat: valorar instalaciones de biomasa` |
| `fix:` | Corrección de un error | `fix: el mínimo de subcontrata se aplicaba por equipo` |
| `normativa:` | Cambio en datos normativos (exige fuente) | `normativa: cotejar tabla 3.1 RITE con BOE 02/08/2022` |
| `datos:` | Catálogo, tiempos o ejemplos | `datos: tiempos de unidades interiores VRF` |
| `docs:` | Documentación | `docs: explicar el cálculo del desplazamiento` |
| `ui:` | Solo interfaz | `ui: filtro por familia en el catálogo` |
| `test:` | Pruebas | `test: caso de instalación >70 kW con temporada de 7 meses` |

Si el cambio resuelve un *issue*, añada en el cuerpo `Cierra #12`.

## Cómo informar de un problema o proponer una mejora

Use **Issues → New issue** y elija la plantilla:

- **Error**: algo no funciona o calcula mal. Incluya el contrato de ejemplo y el resultado esperado.
- **Mejora**: una función nueva.
- **Normativa**: un dato normativo incorrecto o desactualizado. **Obligatorio indicar la fuente oficial** (BOE, artículo, tabla y fecha).

## Comentarios en el código

Mantenix lo mantendrán personas distintas a lo largo de años, así que el código se comenta en abundancia y en español:

- Cada fichero empieza con un comentario que explica **para qué sirve**.
- Cada función, estructura y campo público lleva comentario de documentación (`///` en Rust). El compilador avisa si falta (`#![warn(missing_docs)]`).
- Las reglas normativas citan su origen: `// RITE IT 3.3, tabla 3.1`.
- Los comentarios explican **por qué**, no repiten lo que dice el código.
- Las decisiones de diseño importantes se registran en [docs/decisiones/](docs/decisiones/).

## Antes de abrir la PR

```bash
cargo fmt --all                                  # formato
cargo clippy --workspace --exclude mantenix-app  # avisos
cargo test --workspace --exclude mantenix-app    # pruebas
cd app && npm run build                          # comprueba la interfaz
```

La integración continua de GitHub ejecuta estas mismas comprobaciones en cada PR, en Windows y en Linux. Una PR con fallos no se puede fusionar.

## Cambios en la normativa

1. Edite el CSV en `datos/semilla/normativa/`.
2. Rellene `fuente` y `verificado`. Si el texto está cotejado con el BOE, `verificado=si`.
3. No cambie el `id` de una operación existente: si una norma la sustituye, añada una fila nueva.
4. Explique el cambio en la PR y enlace el BOE.
5. Añada la entrada en el [CHANGELOG](CHANGELOG.md).

Las bases de datos ya creadas no se modifican solas: el cambio se aplica a las bases nuevas o mediante una actualización de normativa (pendiente, ver la hoja de ruta).

## Hoja de ruta

Ver los *milestones* del repositorio. Resumen:

- **0.2**: calibración de tiempos con partes reales; edición del catálogo y de las tarifas de subcontrata desde la app; actualización de normativa en bases existentes.
- **0.3**: oferta para el cliente y contrato desde la app; avisos de renovación por IPC.
- **0.4**: gestión del mantenimiento (visitas, partes, horas reales, certificados RIPCI).
- Más adelante: inventario en tableta Android, fotos, OCR de placas, planos.
