# Datos privados de la empresa

El repositorio es público. **Nunca deben subirse**:

- Tarifas de subcontratistas.
- Costes de personal, márgenes y parámetros reales.
- Contratos, inventarios, nombres o datos de clientes.
- Texto de normas UNE.
- Bases de datos (`*.db`).

`.gitignore` excluye la carpeta `privado/` y todos los ficheros `*.db`. Aun así, **revise siempre la lista de ficheros en GitHub Desktop antes de hacer commit**.

## Organización recomendada

```
privado/                    (fuera de Git; copia de seguridad aparte o repositorio privado)
  empresa/
    catalogo_equipos.csv    catálogo propio (puede partir del de ejemplo)
    tiempos_estandar.csv    tiempos calibrados
    parametros.csv          costes y márgenes reales por año
    tarifa_subcontrata.csv  tarifas de subcontratistas
    empresa.json            razón social, habilitaciones, horario
  contratos/                inventarios reales (JSON) para importar
```

Crear la base de la empresa:

- **Aplicación**: *Base de datos → Crear base con datos de empresa…* y elegir `privado/empresa`.
- **Línea de comandos**: `mantenix --db empresa.db init --empresa privado/empresa`.

A partir de ahí los datos se mantienen **en la base de datos** (con historial de cambios), no en los CSV.

## Formato de los CSV

Separador `;`, codificación UTF-8, primera fila con los nombres de columna. Usar como modelo los de `datos/semilla/ejemplo/`.
