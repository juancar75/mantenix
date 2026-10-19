//! Base de datos SQLite: única fuente de datos de la aplicación.
//!
//! * Los CSV de `datos/semilla/` solo se usan una vez, en [`inicializar`].
//! * Las tablas de usuario (clientes, contratos, inventario) usan identificadores UUID y una
//!   columna `actualizado`, para poder sincronizar varios equipos en el futuro.
//! * **Trazabilidad:** cada alta, cambio o baja en las tablas de datos (normativa, catálogo,
//!   tiempos, parámetros, tarifas, contratos) queda registrada por disparadores en la tabla
//!   `auditoria`, con fecha, usuario, valores anteriores y nuevos. Ver [`fijar_usuario`] y
//!   [`historial`].

use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension};

use crate::datos::{Biblioteca, Equipo, FilaTabla31, Operacion, PartidaSubcontrata, Tiempo};
use crate::modelo::{Contrato, Desplazamiento, Instalacion, LineaEquipo};
use crate::motor::Valoracion;
use crate::presupuesto::{calcular, Presupuesto, ESTADOS, TIPOS_LINEA};
use crate::Error;

const ESQUEMA: &str = r#"
PRAGMA foreign_keys = ON;

-- Biblioteca normativa
CREATE TABLE IF NOT EXISTS operacion (
    id TEXT PRIMARY KEY, texto TEXT NOT NULL, periodicidad TEXT NOT NULL,
    tipo TEXT NOT NULL, referencia TEXT NOT NULL, sistema TEXT NOT NULL DEFAULT '',
    fuente TEXT NOT NULL DEFAULT '', verificado TEXT NOT NULL DEFAULT 'no');
CREATE TABLE IF NOT EXISTS tabla_3_1 (
    id TEXT PRIMARY KEY, equipo TEXT NOT NULL, pn_min_kw REAL, pn_max_kw REAL,
    viviendas_meses REAL NOT NULL, otros_usos_meses REAL NOT NULL);

-- Datos de empresa
CREATE TABLE IF NOT EXISTS equipo (
    codigo TEXT PRIMARY KEY, familia TEXT NOT NULL, descripcion TEXT NOT NULL, unidad TEXT NOT NULL,
    sistema_ripci TEXT NOT NULL DEFAULT '', equipo_tabla_3_1 TEXT NOT NULL DEFAULT '',
    modo_defecto TEXT NOT NULL, partida_subcontrata TEXT NOT NULL DEFAULT '');
CREATE TABLE IF NOT EXISTS tiempo (
    codigo_equipo TEXT NOT NULL REFERENCES equipo(codigo), operacion_id TEXT NOT NULL REFERENCES operacion(id),
    minutos REAL NOT NULL, tecnicos INTEGER NOT NULL DEFAULT 1, consumibles_eur REAL NOT NULL DEFAULT 0,
    estado TEXT NOT NULL DEFAULT '', nota TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (codigo_equipo, operacion_id));
CREATE TABLE IF NOT EXISTS parametro (
    anio INTEGER NOT NULL, nombre TEXT NOT NULL, valor REAL NOT NULL, unidad TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (anio, nombre));
CREATE TABLE IF NOT EXISTS tarifa_subcontrata (
    anio INTEGER NOT NULL, proveedor TEXT NOT NULL, partida TEXT NOT NULL, elemento TEXT NOT NULL,
    unidad TEXT NOT NULL, precio REAL, tipo TEXT NOT NULL,
    PRIMARY KEY (anio, proveedor, partida));

-- Clientes y contratos
CREATE TABLE IF NOT EXISTS cliente (
    id TEXT PRIMARY KEY, razon_social TEXT NOT NULL, cif TEXT NOT NULL DEFAULT '',
    domicilio TEXT NOT NULL DEFAULT '', actualizado TEXT NOT NULL DEFAULT (datetime('now')));
CREATE TABLE IF NOT EXISTS contrato (
    id TEXT PRIMARY KEY, cliente_id TEXT REFERENCES cliente(id), nombre TEXT NOT NULL UNIQUE,
    anio INTEGER NOT NULL, uso TEXT NOT NULL DEFAULT 'otros', visitas_contratadas INTEGER,
    km_ida REAL NOT NULL DEFAULT 0, minutos_ida REAL NOT NULL DEFAULT 0,
    margen_pct REAL, precio_actual_anual REAL,
    actualizado TEXT NOT NULL DEFAULT (datetime('now')));
CREATE TABLE IF NOT EXISTS instalacion (
    id TEXT PRIMARY KEY, contrato_id TEXT NOT NULL REFERENCES contrato(id) ON DELETE CASCADE,
    orden INTEGER NOT NULL, familia TEXT NOT NULL, titulo TEXT NOT NULL DEFAULT '',
    potencia_kw REAL, temporada_meses REAL NOT NULL DEFAULT 12,
    actualizado TEXT NOT NULL DEFAULT (datetime('now')));
CREATE TABLE IF NOT EXISTS linea_equipo (
    id TEXT PRIMARY KEY, instalacion_id TEXT NOT NULL REFERENCES instalacion(id) ON DELETE CASCADE,
    orden INTEGER NOT NULL, codigo TEXT NOT NULL REFERENCES equipo(codigo), cantidad REAL NOT NULL,
    descripcion TEXT NOT NULL DEFAULT '', modo TEXT,
    actualizado TEXT NOT NULL DEFAULT (datetime('now')));

-- Datos de la empresa mantenedora (JSON: razón social, habilitaciones, horario…)
CREATE TABLE IF NOT EXISTS empresa (id INTEGER PRIMARY KEY CHECK (id = 1), json TEXT NOT NULL);

-- Trazabilidad
CREATE TABLE IF NOT EXISTS sesion (usuario TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS auditoria (
    id INTEGER PRIMARY KEY AUTOINCREMENT, fecha TEXT NOT NULL DEFAULT (datetime('now')),
    usuario TEXT, tabla TEXT NOT NULL, accion TEXT NOT NULL, antes TEXT, despues TEXT);

-- Presupuestos de reparación y pequeñas obras (el documento completo va en `json`)
CREATE TABLE IF NOT EXISTS presupuesto (
    id TEXT PRIMARY KEY, numero TEXT NOT NULL UNIQUE, anio INTEGER NOT NULL, fecha TEXT NOT NULL,
    cliente TEXT NOT NULL, titulo TEXT NOT NULL, estado TEXT NOT NULL,
    base REAL NOT NULL, total REAL NOT NULL, json TEXT NOT NULL,
    actualizado TEXT NOT NULL DEFAULT (datetime('now')));

-- Valoraciones guardadas (reproducibles: se guarda el resultado completo)
CREATE TABLE IF NOT EXISTS valoracion (
    id TEXT PRIMARY KEY, contrato_id TEXT NOT NULL REFERENCES contrato(id) ON DELETE CASCADE,
    anio INTEGER NOT NULL, fecha TEXT NOT NULL DEFAULT (datetime('now')),
    coste_total REAL NOT NULL, precio_anual REAL NOT NULL, json TEXT NOT NULL);
"#;

fn e(err: rusqlite::Error) -> Error {
    Error::Db(err.to_string())
}

fn uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Tablas con registro de cambios y sus columnas.
const AUDITADAS: &[(&str, &[&str])] = &[
    (
        "operacion",
        &[
            "id",
            "texto",
            "periodicidad",
            "tipo",
            "referencia",
            "sistema",
            "fuente",
            "verificado",
        ],
    ),
    (
        "tabla_3_1",
        &[
            "id",
            "equipo",
            "pn_min_kw",
            "pn_max_kw",
            "viviendas_meses",
            "otros_usos_meses",
        ],
    ),
    (
        "equipo",
        &[
            "codigo",
            "familia",
            "descripcion",
            "unidad",
            "sistema_ripci",
            "equipo_tabla_3_1",
            "modo_defecto",
            "partida_subcontrata",
        ],
    ),
    (
        "tiempo",
        &[
            "codigo_equipo",
            "operacion_id",
            "minutos",
            "tecnicos",
            "consumibles_eur",
            "estado",
            "nota",
        ],
    ),
    ("parametro", &["anio", "nombre", "valor", "unidad"]),
    (
        "tarifa_subcontrata",
        &["anio", "proveedor", "partida", "elemento", "unidad", "precio", "tipo"],
    ),
    ("cliente", &["id", "razon_social", "cif", "domicilio"]),
    (
        "contrato",
        &[
            "id",
            "cliente_id",
            "nombre",
            "anio",
            "uso",
            "visitas_contratadas",
            "km_ida",
            "minutos_ida",
            "margen_pct",
            "precio_actual_anual",
        ],
    ),
    (
        "linea_equipo",
        &["id", "instalacion_id", "codigo", "cantidad", "descripcion", "modo"],
    ),
    ("empresa", &["id", "json"]),
    (
        "presupuesto",
        &[
            "id", "numero", "fecha", "cliente", "titulo", "estado", "base", "total", "json",
        ],
    ),
];

/// Genera los disparadores de auditoría (alta, cambio y baja) de una tabla.
fn disparadores(tabla: &str, cols: &[&str]) -> String {
    let obj = |p: &str| {
        format!(
            "json_object({})",
            cols.iter()
                .map(|c| format!("'{c}', {p}.{c}"))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    let usuario = "(SELECT usuario FROM sesion LIMIT 1)";
    format!(
        "CREATE TRIGGER IF NOT EXISTS aud_{t}_i AFTER INSERT ON {t} BEGIN \
           INSERT INTO auditoria (usuario, tabla, accion, despues) VALUES ({u}, '{t}', 'alta', {n}); END;
         CREATE TRIGGER IF NOT EXISTS aud_{t}_u AFTER UPDATE ON {t} BEGIN \
           INSERT INTO auditoria (usuario, tabla, accion, antes, despues) VALUES ({u}, '{t}', 'cambio', {o}, {n}); END;
         CREATE TRIGGER IF NOT EXISTS aud_{t}_d AFTER DELETE ON {t} BEGIN \
           INSERT INTO auditoria (usuario, tabla, accion, antes) VALUES ({u}, '{t}', 'baja', {o}); END;",
        t = tabla,
        u = usuario,
        n = obj("NEW"),
        o = obj("OLD")
    )
}

/// Abre (o crea) la base de datos y asegura el esquema y los disparadores de auditoría.
pub fn abrir(ruta: &Path) -> Result<Connection, Error> {
    let c = Connection::open(ruta).map_err(e)?;
    c.execute_batch(ESQUEMA).map_err(e)?;
    for (t, cols) in AUDITADAS {
        c.execute_batch(&disparadores(t, cols)).map_err(e)?;
    }
    // El usuario se identifica en cada sesión: no se hereda el de una sesión anterior.
    c.execute("DELETE FROM sesion", []).map_err(e)?;
    Ok(c)
}

/// Identifica a quién se atribuyen los cambios siguientes en la tabla `auditoria`.
pub fn fijar_usuario(c: &Connection, usuario: &str) -> Result<(), Error> {
    c.execute("DELETE FROM sesion", []).map_err(e)?;
    c.execute("INSERT INTO sesion (usuario) VALUES (?1)", [usuario])
        .map_err(e)?;
    Ok(())
}

/// Entrada del registro de cambios.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Cambio {
    /// Número de orden.
    pub id: i64,
    /// Fecha y hora UTC.
    pub fecha: String,
    /// Usuario que hizo el cambio (si se fijó con [`fijar_usuario`]).
    pub usuario: Option<String>,
    /// Tabla afectada.
    pub tabla: String,
    /// `alta`, `cambio` o `baja`.
    pub accion: String,
    /// Fila antes del cambio (JSON).
    pub antes: Option<String>,
    /// Fila después del cambio (JSON).
    pub despues: Option<String>,
}

/// Últimos cambios registrados, opcionalmente filtrados por tabla.
pub fn historial(c: &Connection, tabla: Option<&str>, limite: u32) -> Result<Vec<Cambio>, Error> {
    let mut s = c
        .prepare(
            "SELECT id,fecha,usuario,tabla,accion,antes,despues FROM auditoria
         WHERE (?1 IS NULL OR tabla = ?1) ORDER BY id DESC LIMIT ?2",
        )
        .map_err(e)?;
    let v = s
        .query_map(params![tabla, limite], |r| {
            Ok(Cambio {
                id: r.get(0)?,
                fecha: r.get(1)?,
                usuario: r.get(2)?,
                tabla: r.get(3)?,
                accion: r.get(4)?,
                antes: r.get(5)?,
                despues: r.get(6)?,
            })
        })
        .map_err(e)?
        .collect::<Result<_, _>>()
        .map_err(e)?;
    Ok(v)
}

/// Carga la semilla CSV en una base vacía. No hace nada si ya hay datos.
///
/// La carga inicial también queda en `auditoria`, atribuida al usuario `semilla`.
pub fn inicializar(c: &mut Connection, normativa: &Path, empresa: &Path) -> Result<bool, Error> {
    let n: i64 = c
        .query_row("SELECT count(*) FROM operacion", [], |r| r.get(0))
        .map_err(e)?;
    if n > 0 {
        return Ok(false);
    }
    fijar_usuario(c, "semilla")?;
    let b = Biblioteca::desde_csv(normativa, empresa)?;
    let tx = c.transaction().map_err(e)?;
    for o in b.operaciones.values() {
        tx.execute(
            "INSERT INTO operacion VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                o.id,
                o.texto,
                o.periodicidad,
                o.tipo,
                o.referencia,
                o.sistema,
                o.fuente,
                o.verificado
            ],
        )
        .map_err(e)?;
    }
    for f in &b.tabla_3_1 {
        tx.execute(
            "INSERT INTO tabla_3_1 VALUES (?1,?2,?3,?4,?5,?6)",
            params![
                f.id,
                f.equipo,
                f.pn_min_kw,
                f.pn_max_kw,
                f.viviendas_meses,
                f.otros_usos_meses
            ],
        )
        .map_err(e)?;
    }
    for q in b.catalogo.values() {
        tx.execute(
            "INSERT INTO equipo VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                q.codigo,
                q.familia,
                q.descripcion,
                q.unidad,
                q.sistema_ripci,
                q.equipo_tabla_3_1,
                q.modo_defecto,
                q.partida_subcontrata
            ],
        )
        .map_err(e)?;
    }
    for t in &b.tiempos {
        tx.execute(
            "INSERT INTO tiempo VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![
                t.codigo_equipo,
                t.operacion_id,
                t.minutos,
                t.tecnicos,
                t.consumibles_eur,
                t.estado,
                t.nota
            ],
        )
        .map_err(e)?;
    }
    for ((anio, nombre), valor) in &b.parametros {
        tx.execute(
            "INSERT INTO parametro (anio, nombre, valor) VALUES (?1,?2,?3)",
            params![anio, nombre, valor],
        )
        .map_err(e)?;
    }
    for p in &b.subcontrata {
        tx.execute(
            "INSERT INTO tarifa_subcontrata VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![p.anio, p.proveedor, p.partida, p.elemento, p.unidad, p.precio, p.tipo],
        )
        .map_err(e)?;
    }
    let emp = empresa.join("empresa.json");
    if emp.exists() {
        let txt =
            std::fs::read_to_string(&emp).map_err(|x| Error::Fichero(emp.display().to_string(), x.to_string()))?;
        tx.execute("INSERT INTO empresa (id, json) VALUES (1, ?1)", [txt])
            .map_err(e)?;
    }
    tx.commit().map_err(e)?;
    c.execute("DELETE FROM sesion", []).map_err(e)?;
    Ok(true)
}

/// Datos de la empresa mantenedora (JSON). Vacío si no se han cargado.
pub fn empresa(c: &Connection) -> Result<serde_json::Value, Error> {
    let t: Option<String> = c
        .query_row("SELECT json FROM empresa WHERE id=1", [], |r| r.get(0))
        .optional()
        .map_err(e)?;
    Ok(t.and_then(|x| serde_json::from_str(&x).ok())
        .unwrap_or_else(|| serde_json::json!({"razon_social": ""})))
}

/// Guarda los datos de la empresa mantenedora.
pub fn fijar_empresa(c: &Connection, datos: &serde_json::Value) -> Result<(), Error> {
    c.execute(
        "INSERT INTO empresa (id, json) VALUES (1, ?1) ON CONFLICT(id) DO UPDATE SET json=excluded.json",
        [datos.to_string()],
    )
    .map_err(e)?;
    Ok(())
}

impl Biblioteca {
    /// Carga la biblioteca completa desde SQLite.
    pub fn desde_db(c: &Connection) -> Result<Self, Error> {
        let mut b = Biblioteca::default();
        let mut s = c.prepare("SELECT codigo,familia,descripcion,unidad,sistema_ripci,equipo_tabla_3_1,modo_defecto,partida_subcontrata FROM equipo").map_err(e)?;
        for q in s
            .query_map([], |r| {
                Ok(Equipo {
                    codigo: r.get(0)?,
                    familia: r.get(1)?,
                    descripcion: r.get(2)?,
                    unidad: r.get(3)?,
                    sistema_ripci: r.get(4)?,
                    equipo_tabla_3_1: r.get(5)?,
                    modo_defecto: r.get(6)?,
                    partida_subcontrata: r.get(7)?,
                })
            })
            .map_err(e)?
        {
            let q = q.map_err(e)?;
            b.catalogo.insert(q.codigo.clone(), q);
        }

        let mut s = c.prepare("SELECT codigo_equipo,operacion_id,minutos,tecnicos,consumibles_eur,estado,nota FROM tiempo ORDER BY rowid").map_err(e)?;
        b.tiempos = s
            .query_map([], |r| {
                Ok(Tiempo {
                    codigo_equipo: r.get(0)?,
                    operacion_id: r.get(1)?,
                    minutos: r.get(2)?,
                    tecnicos: r.get(3)?,
                    consumibles_eur: r.get(4)?,
                    estado: r.get(5)?,
                    nota: r.get(6)?,
                })
            })
            .map_err(e)?
            .collect::<Result<_, _>>()
            .map_err(e)?;

        let mut s = c
            .prepare("SELECT id,texto,periodicidad,tipo,referencia,sistema,fuente,verificado FROM operacion")
            .map_err(e)?;
        for o in s
            .query_map([], |r| {
                Ok(Operacion {
                    id: r.get(0)?,
                    texto: r.get(1)?,
                    periodicidad: r.get(2)?,
                    tipo: r.get(3)?,
                    referencia: r.get(4)?,
                    sistema: r.get(5)?,
                    fuente: r.get(6)?,
                    verificado: r.get(7)?,
                })
            })
            .map_err(e)?
        {
            let o = o.map_err(e)?;
            b.operaciones.insert(o.id.clone(), o);
        }

        let mut s = c
            .prepare("SELECT id,equipo,pn_min_kw,pn_max_kw,viviendas_meses,otros_usos_meses FROM tabla_3_1")
            .map_err(e)?;
        b.tabla_3_1 = s
            .query_map([], |r| {
                Ok(FilaTabla31 {
                    id: r.get(0)?,
                    equipo: r.get(1)?,
                    pn_min_kw: r.get(2)?,
                    pn_max_kw: r.get(3)?,
                    viviendas_meses: r.get(4)?,
                    otros_usos_meses: r.get(5)?,
                })
            })
            .map_err(e)?
            .collect::<Result<_, _>>()
            .map_err(e)?;

        let mut s = c
            .prepare("SELECT anio,proveedor,partida,elemento,unidad,precio,tipo FROM tarifa_subcontrata")
            .map_err(e)?;
        b.subcontrata = s
            .query_map([], |r| {
                Ok(PartidaSubcontrata {
                    anio: r.get(0)?,
                    proveedor: r.get(1)?,
                    partida: r.get(2)?,
                    elemento: r.get(3)?,
                    unidad: r.get(4)?,
                    precio: r.get(5)?,
                    tipo: r.get(6)?,
                })
            })
            .map_err(e)?
            .collect::<Result<_, _>>()
            .map_err(e)?;

        let mut s = c.prepare("SELECT anio,nombre,valor FROM parametro").map_err(e)?;
        for p in s
            .query_map([], |r| {
                Ok(((r.get::<_, u32>(0)?, r.get::<_, String>(1)?), r.get::<_, f64>(2)?))
            })
            .map_err(e)?
        {
            let (k, v) = p.map_err(e)?;
            b.parametros.insert(k, v);
        }
        Ok(b)
    }
}

/// Guarda (o sustituye, por nombre) un contrato con su inventario. Devuelve su id.
pub fn guardar_contrato(c: &mut Connection, k: &Contrato) -> Result<String, Error> {
    let tx = c.transaction().map_err(e)?;
    let existente: Option<String> = tx
        .query_row("SELECT id FROM contrato WHERE nombre=?1", [&k.nombre], |r| r.get(0))
        .optional()
        .map_err(e)?;
    let id = existente.unwrap_or_else(uuid);
    tx.execute("DELETE FROM instalacion WHERE contrato_id=?1", [&id])
        .map_err(e)?;
    tx.execute(
        "INSERT INTO contrato (id,nombre,anio,uso,visitas_contratadas,km_ida,minutos_ida,margen_pct,precio_actual_anual)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)
         ON CONFLICT(id) DO UPDATE SET anio=excluded.anio, uso=excluded.uso,
           visitas_contratadas=excluded.visitas_contratadas, km_ida=excluded.km_ida, minutos_ida=excluded.minutos_ida,
           margen_pct=excluded.margen_pct, precio_actual_anual=excluded.precio_actual_anual, actualizado=datetime('now')",
        params![id, k.nombre, k.anio, k.uso, k.visitas_contratadas, k.desplazamiento.km_ida,
                k.desplazamiento.minutos_ida, k.margen_pct, k.precio_actual_anual]).map_err(e)?;
    for (i, ins) in k.instalaciones.iter().enumerate() {
        let iid = uuid();
        tx.execute("INSERT INTO instalacion (id,contrato_id,orden,familia,titulo,potencia_kw,temporada_meses) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![iid, id, i as i64, ins.familia, ins.titulo, ins.potencia_kw, ins.temporada_meses]).map_err(e)?;
        for (j, l) in ins.equipos.iter().enumerate() {
            tx.execute("INSERT INTO linea_equipo (id,instalacion_id,orden,codigo,cantidad,descripcion,modo) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                params![uuid(), iid, j as i64, l.codigo, l.cantidad, l.descripcion, l.modo]).map_err(e)?;
        }
    }
    tx.commit().map_err(e)?;
    Ok(id)
}

/// Carga un contrato por nombre.
pub fn cargar_contrato(c: &Connection, nombre: &str) -> Result<(String, Contrato), Error> {
    let (id, mut k) = c.query_row(
        "SELECT id,nombre,anio,uso,visitas_contratadas,km_ida,minutos_ida,margen_pct,precio_actual_anual FROM contrato WHERE nombre=?1",
        [nombre], |r| Ok((r.get::<_, String>(0)?, Contrato {
            nombre: r.get(1)?, anio: r.get(2)?, uso: r.get(3)?, visitas_contratadas: r.get(4)?,
            desplazamiento: Desplazamiento { km_ida: r.get(5)?, minutos_ida: r.get(6)? },
            margen_pct: r.get(7)?, precio_actual_anual: r.get(8)?, instalaciones: vec![],
        }))).optional().map_err(e)?.ok_or_else(|| Error::NoEncontrado(nombre.to_string()))?;
    let mut si = c
        .prepare(
            "SELECT id,familia,titulo,potencia_kw,temporada_meses FROM instalacion WHERE contrato_id=?1 ORDER BY orden",
        )
        .map_err(e)?;
    let mut sl = c
        .prepare("SELECT codigo,cantidad,descripcion,modo FROM linea_equipo WHERE instalacion_id=?1 ORDER BY orden")
        .map_err(e)?;
    let filas: Vec<(String, Instalacion)> = si
        .query_map([&id], |r| {
            Ok((
                r.get(0)?,
                Instalacion {
                    familia: r.get(1)?,
                    titulo: r.get(2)?,
                    potencia_kw: r.get(3)?,
                    temporada_meses: r.get(4)?,
                    equipos: vec![],
                },
            ))
        })
        .map_err(e)?
        .collect::<Result<_, _>>()
        .map_err(e)?;
    for (iid, mut ins) in filas {
        ins.equipos = sl
            .query_map([&iid], |r| {
                Ok(LineaEquipo {
                    codigo: r.get(0)?,
                    cantidad: r.get(1)?,
                    descripcion: r.get(2)?,
                    modo: r.get(3)?,
                })
            })
            .map_err(e)?
            .collect::<Result<_, _>>()
            .map_err(e)?;
        k.instalaciones.push(ins);
    }
    Ok((id, k))
}

/// Contratos guardados: nombre, año y último precio valorado.
pub fn listar_contratos(c: &Connection) -> Result<Vec<(String, u32, Option<f64>)>, Error> {
    let mut s = c.prepare(
        "SELECT c.nombre, c.anio, (SELECT precio_anual FROM valoracion v WHERE v.contrato_id=c.id ORDER BY fecha DESC LIMIT 1)
         FROM contrato c ORDER BY c.nombre").map_err(e)?;
    let v = s
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .map_err(e)?
        .collect::<Result<_, _>>()
        .map_err(e)?;
    Ok(v)
}

/// Guarda el resultado completo de una valoración (para reproducirla y compararla en renovaciones).
pub fn guardar_valoracion(c: &Connection, contrato_id: &str, v: &Valoracion) -> Result<String, Error> {
    let id = uuid();
    let json = serde_json::to_string(v).map_err(|x| Error::Db(x.to_string()))?;
    c.execute(
        "INSERT INTO valoracion (id,contrato_id,anio,coste_total,precio_anual,json) VALUES (?1,?2,?3,?4,?5,?6)",
        params![
            id,
            contrato_id,
            v.anio,
            v.totales.coste_total,
            v.totales.precio_anual,
            json
        ],
    )
    .map_err(e)?;
    Ok(id)
}

/// Fija o cambia un parámetro de un año (p. ej. coste hora 2027).
pub fn fijar_parametro(c: &Connection, anio: u32, nombre: &str, valor: f64) -> Result<(), Error> {
    c.execute(
        "INSERT INTO parametro (anio,nombre,valor) VALUES (?1,?2,?3)
               ON CONFLICT(anio,nombre) DO UPDATE SET valor=excluded.valor",
        params![anio, nombre, valor],
    )
    .map_err(e)?;
    Ok(())
}

/// Cambia el tiempo estándar de una operación sobre un equipo (queda en `auditoria`).
pub fn fijar_tiempo(
    c: &Connection,
    codigo_equipo: &str,
    operacion_id: &str,
    minutos: f64,
    tecnicos: u32,
) -> Result<(), Error> {
    c.execute(
        "INSERT INTO tiempo (codigo_equipo, operacion_id, minutos, tecnicos, estado) VALUES (?1,?2,?3,?4,'revisado')
         ON CONFLICT(codigo_equipo, operacion_id) DO UPDATE SET minutos=excluded.minutos, tecnicos=excluded.tecnicos, estado='revisado'",
        params![codigo_equipo, operacion_id, minutos, tecnicos]).map_err(e)?;
    Ok(())
}

/// Parámetros de un año (los que faltan se heredan del año anterior al valorar).
pub fn parametros_de(c: &Connection, anio: u32) -> Result<Vec<(String, f64)>, Error> {
    let mut s = c
        .prepare("SELECT nombre, valor FROM parametro WHERE anio=?1 ORDER BY nombre")
        .map_err(e)?;
    let v = s
        .query_map([anio], |r| Ok((r.get(0)?, r.get(1)?)))
        .map_err(e)?
        .collect::<Result<_, _>>()
        .map_err(e)?;
    Ok(v)
}

/// Elimina un contrato con su inventario y sus valoraciones.
pub fn borrar_contrato(c: &Connection, nombre: &str) -> Result<(), Error> {
    c.execute("PRAGMA foreign_keys = ON", []).map_err(e)?;
    let n = c.execute("DELETE FROM contrato WHERE nombre=?1", [nombre]).map_err(e)?;
    if n == 0 {
        return Err(Error::NoEncontrado(nombre.to_string()));
    }
    Ok(())
}

/// Resumen de un presupuesto para listados.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ResumenPresupuesto {
    /// Número (`P-AAAA-NNN`).
    pub numero: String,
    /// Fecha de emisión.
    pub fecha: String,
    /// Cliente.
    pub cliente: String,
    /// Título del trabajo.
    pub titulo: String,
    /// Estado.
    pub estado: String,
    /// Total con IVA (€).
    pub total: f64,
}

/// Siguiente número libre del año: `P-2026-001`, `P-2026-002`…
pub fn siguiente_numero(c: &Connection, anio: u32) -> Result<String, Error> {
    let prefijo = format!("P-{anio}-");
    let ultimo: Option<i64> = c
        .query_row(
            "SELECT MAX(CAST(substr(numero, ?2) AS INTEGER)) FROM presupuesto WHERE numero LIKE ?1 || '%'",
            params![prefijo, prefijo.len() as i64 + 1],
            |r| r.get(0),
        )
        .map_err(e)?;
    Ok(format!("{prefijo}{:03}", ultimo.unwrap_or(0) + 1))
}

/// Guarda un presupuesto. Si no tiene número se le asigna el siguiente del año; si ya existe
/// uno con ese número, se sustituye. Devuelve el número.
pub fn guardar_presupuesto(c: &Connection, p: &Presupuesto) -> Result<String, Error> {
    let invalido = |m: String| Error::Db(m);
    if p.cliente.trim().is_empty() || p.titulo.trim().is_empty() {
        return Err(invalido("El presupuesto necesita cliente y título".into()));
    }
    if !ESTADOS.contains(&p.estado.as_str()) {
        return Err(invalido(format!("Estado no válido: {}", p.estado)));
    }
    if let Some(l) = p.lineas.iter().find(|l| !TIPOS_LINEA.contains(&l.tipo.as_str())) {
        return Err(invalido(format!("Tipo de línea no válido: {}", l.tipo)));
    }
    let mut p = p.clone();
    if p.numero.trim().is_empty() {
        p.numero = siguiente_numero(c, p.anio())?;
    }
    let t = calcular(&p);
    let json = serde_json::to_string(&p).map_err(|x| Error::Db(x.to_string()))?;
    let id: Option<String> = c
        .query_row("SELECT id FROM presupuesto WHERE numero=?1", [&p.numero], |r| r.get(0))
        .optional()
        .map_err(e)?;
    match id {
        Some(id) => c.execute(
            "UPDATE presupuesto SET anio=?2, fecha=?3, cliente=?4, titulo=?5, estado=?6, base=?7, total=?8, json=?9,
             actualizado=datetime('now') WHERE id=?1",
            params![
                id,
                p.anio(),
                p.fecha,
                p.cliente,
                p.titulo,
                p.estado,
                t.base,
                t.total,
                json
            ],
        ),
        None => c.execute(
            "INSERT INTO presupuesto (id, numero, anio, fecha, cliente, titulo, estado, base, total, json)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            params![
                uuid(),
                p.numero,
                p.anio(),
                p.fecha,
                p.cliente,
                p.titulo,
                p.estado,
                t.base,
                t.total,
                json
            ],
        ),
    }
    .map_err(e)?;
    Ok(p.numero)
}

/// Carga un presupuesto por su número.
pub fn cargar_presupuesto(c: &Connection, numero: &str) -> Result<Presupuesto, Error> {
    let json: String = c
        .query_row("SELECT json FROM presupuesto WHERE numero=?1", [numero], |r| r.get(0))
        .optional()
        .map_err(e)?
        .ok_or_else(|| Error::NoEncontrado(numero.to_string()))?;
    serde_json::from_str(&json).map_err(|x| Error::Db(x.to_string()))
}

/// Presupuestos guardados, del más reciente al más antiguo.
pub fn listar_presupuestos(c: &Connection) -> Result<Vec<ResumenPresupuesto>, Error> {
    let mut s = c
        .prepare(
            "SELECT numero, fecha, cliente, titulo, estado, total FROM presupuesto ORDER BY fecha DESC, numero DESC",
        )
        .map_err(e)?;
    let v = s
        .query_map([], |r| {
            Ok(ResumenPresupuesto {
                numero: r.get(0)?,
                fecha: r.get(1)?,
                cliente: r.get(2)?,
                titulo: r.get(3)?,
                estado: r.get(4)?,
                total: r.get(5)?,
            })
        })
        .map_err(e)?
        .collect::<Result<_, _>>()
        .map_err(e)?;
    Ok(v)
}

/// Elimina un presupuesto (queda constancia en `auditoria`).
pub fn borrar_presupuesto(c: &Connection, numero: &str) -> Result<(), Error> {
    if c.execute("DELETE FROM presupuesto WHERE numero=?1", [numero])
        .map_err(e)?
        == 0
    {
        return Err(Error::NoEncontrado(numero.to_string()));
    }
    Ok(())
}
