# Trazabilidad

Mantenix registra el origen y la historia de cada dato en tres niveles.

## 1. Código y datos de semilla: Git y GitHub

- Cada cambio es un *commit* con autor, fecha y motivo ([formato](../CONTRIBUTING.md#formato-de-los-mensajes-de-commit)).
- Cada cambio pasa por una *pull request* revisada, enlazada al *issue* que lo motiva.
- Las versiones publicadas se marcan con etiquetas (`v0.1.0`) y se describen en el [CHANGELOG](../CHANGELOG.md).
- Las decisiones de diseño se registran en [decisiones/](decisiones/).

## 2. Datos normativos: fuente y cotejo

Cada operación lleva `fuente` (documento del que procede) y `verificado` (estado de cotejo con el texto oficial). Ver [normativa.md](normativa.md).

## 3. Datos de trabajo: registro de cambios en SQLite

Disparadores de SQLite escriben en la tabla `auditoria` cada **alta, cambio o baja** en:

`operacion`, `tabla_3_1`, `equipo`, `tiempo`, `parametro`, `tarifa_subcontrata`, `empresa`, `cliente`, `contrato`, `linea_equipo`.

Cada registro guarda: fecha (UTC), **usuario**, tabla, acción, **valor anterior** y **valor nuevo** (JSON).

- El usuario se identifica en *Base de datos → Usuario* (o `--usuario` en la línea de comandos).
- La carga inicial queda atribuida a `semilla`.
- Al estar en disparadores, ninguna parte de la aplicación puede saltarse el registro.
- Se consulta en *Historial de cambios* o con `mantenix historial`.

## 4. Valoraciones

Cada vez que se valora un contrato se guarda el **resultado completo** (tabla `valoracion`, JSON), con los parámetros usados. Así se puede justificar años después por qué se ofertó un precio, aunque hayan cambiado los tiempos o los costes.
