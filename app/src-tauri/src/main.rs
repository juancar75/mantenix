//! Aplicación de escritorio de Mantenix (Tauri 2).
//!
//! Este fichero solo expone a la interfaz (Vue, carpeta `app/src`) las funciones del núcleo
//! `mantenix-core` y de `mantenix-informes`. **No contiene lógica de cálculo**: cualquier regla
//! de negocio va en el núcleo, donde tiene pruebas automáticas.
//!
//! Cada función marcada con `#[tauri::command]` se llama desde TypeScript con `invoke("nombre", {…})`
//! (ver `app/src/api.ts`, que tiene los tipos de cada llamada).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use mantenix_core::{db, modelo::Contrato, tarifa, valorar, Biblioteca, Valoracion};
use rusqlite::Connection;
use serde::Serialize;
use tauri::{Manager, State};

/// Estado compartido: conexión abierta y ruta del fichero.
struct Base {
    conexion: Mutex<Connection>,
    ruta: Mutex<PathBuf>,
}

/// Los errores se devuelven a la interfaz como texto, para mostrarlos al usuario.
type R<T> = Result<T, String>;

fn txt<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

/// Carpeta de la semilla: recursos instalados con la app o, en desarrollo, la del repositorio.
fn carpeta_semilla(app: &tauri::AppHandle) -> PathBuf {
    if let Ok(r) = app.path().resource_dir() {
        let p = r.join("semilla");
        if p.exists() {
            return p;
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../datos/semilla")
}

/// Abre la base y, si está vacía, la crea con la normativa y los datos de empresa indicados.
fn abrir_o_crear(ruta: &Path, normativa: &Path, empresa: &Path) -> R<Connection> {
    let mut c = db::abrir(ruta).map_err(txt)?;
    db::inicializar(&mut c, normativa, empresa).map_err(txt)?;
    Ok(c)
}

// ───────────────────────────── Base de datos ─────────────────────────────

/// Información general para la barra de estado.
#[derive(Serialize)]
struct Estado {
    ruta: String,
    usuario: Option<String>,
    contratos: usize,
    empresa: String,
}

#[tauri::command]
fn estado(base: State<Base>) -> R<Estado> {
    let c = base.conexion.lock().unwrap();
    let usuario: Option<String> = c.query_row("SELECT usuario FROM sesion LIMIT 1", [], |r| r.get(0)).ok();
    let empresa = db::empresa(&c).map_err(txt)?;
    Ok(Estado {
        ruta: base.ruta.lock().unwrap().display().to_string(),
        usuario,
        contratos: db::listar_contratos(&c).map_err(txt)?.len(),
        empresa: empresa["razon_social"].as_str().unwrap_or_default().to_string(),
    })
}

/// Identifica al usuario: sus cambios quedan registrados a su nombre.
#[tauri::command]
fn fijar_usuario(base: State<Base>, usuario: String) -> R<()> {
    db::fijar_usuario(&base.conexion.lock().unwrap(), &usuario).map_err(txt)
}

/// Abre otra base de datos (p. ej. la de la empresa en una carpeta compartida).
#[tauri::command]
fn abrir_base(app: tauri::AppHandle, base: State<Base>, ruta: String) -> R<()> {
    let s = carpeta_semilla(&app);
    let c = abrir_o_crear(Path::new(&ruta), &s.join("normativa"), &s.join("ejemplo"))?;
    *base.conexion.lock().unwrap() = c;
    *base.ruta.lock().unwrap() = PathBuf::from(ruta);
    Ok(())
}

/// Crea una base nueva con los datos de empresa de una carpeta (p. ej. `privado/empresa`).
#[tauri::command]
fn crear_base(app: tauri::AppHandle, base: State<Base>, ruta: String, carpeta_empresa: String) -> R<()> {
    if Path::new(&ruta).exists() {
        return Err(format!("Ya existe {ruta}; elija otro nombre."));
    }
    let s = carpeta_semilla(&app);
    let c = abrir_o_crear(Path::new(&ruta), &s.join("normativa"), Path::new(&carpeta_empresa))?;
    *base.conexion.lock().unwrap() = c;
    *base.ruta.lock().unwrap() = PathBuf::from(ruta);
    Ok(())
}

// ───────────────────────────── Contratos ─────────────────────────────

/// Fila de la lista de contratos.
#[derive(Serialize)]
struct FilaContrato {
    nombre: String,
    anio: u32,
    precio: Option<f64>,
}

#[tauri::command]
fn listar_contratos(base: State<Base>) -> R<Vec<FilaContrato>> {
    let c = base.conexion.lock().unwrap();
    Ok(db::listar_contratos(&c)
        .map_err(txt)?
        .into_iter()
        .map(|(nombre, anio, precio)| FilaContrato { nombre, anio, precio })
        .collect())
}

#[tauri::command]
fn cargar_contrato(base: State<Base>, nombre: String) -> R<Contrato> {
    Ok(db::cargar_contrato(&base.conexion.lock().unwrap(), &nombre)
        .map_err(txt)?
        .1)
}

#[tauri::command]
fn guardar_contrato(base: State<Base>, contrato: Contrato) -> R<String> {
    if contrato.nombre.trim().is_empty() {
        return Err("El contrato necesita un nombre.".into());
    }
    db::guardar_contrato(&mut base.conexion.lock().unwrap(), &contrato).map_err(txt)
}

#[tauri::command]
fn borrar_contrato(base: State<Base>, nombre: String) -> R<()> {
    db::borrar_contrato(&base.conexion.lock().unwrap(), &nombre).map_err(txt)
}

/// Valora un contrato guardado y guarda la valoración (queda en el historial del contrato).
#[tauri::command]
fn valorar_contrato(base: State<Base>, nombre: String) -> R<Valoracion> {
    let c = base.conexion.lock().unwrap();
    let b = Biblioteca::desde_db(&c).map_err(txt)?;
    let (id, k) = db::cargar_contrato(&c, &nombre).map_err(txt)?;
    let v = valorar(&b, &k).map_err(txt)?;
    db::guardar_valoracion(&c, &id, &v).map_err(txt)?;
    Ok(v)
}

/// Genera la hoja de costes en PDF en la ruta elegida por el usuario.
#[tauri::command]
fn pdf_hoja_costes(base: State<Base>, nombre: String, ruta: String) -> R<()> {
    let c = base.conexion.lock().unwrap();
    let b = Biblioteca::desde_db(&c).map_err(txt)?;
    let (_, k) = db::cargar_contrato(&c, &nombre).map_err(txt)?;
    let v = valorar(&b, &k).map_err(txt)?;
    let pdf = mantenix_informes::hoja_costes(&v, &db::empresa(&c).map_err(txt)?).map_err(txt)?;
    std::fs::write(ruta, pdf).map_err(txt)
}

// ───────────────────────────── Catálogo y tiempos ─────────────────────────────

/// Tiempo estándar con el texto de su operación, para editarlo.
#[derive(Serialize)]
struct TiempoVista {
    operacion_id: String,
    texto: String,
    periodicidad: String,
    referencia: String,
    fuente: String,
    verificado: String,
    minutos: f64,
    tecnicos: u32,
    estado: String,
}

/// Equipo del catálogo con sus tiempos.
#[derive(Serialize)]
struct EquipoVista {
    codigo: String,
    familia: String,
    descripcion: String,
    unidad: String,
    modo_defecto: String,
    partida_subcontrata: String,
    tiempos: Vec<TiempoVista>,
}

#[tauri::command]
fn catalogo(base: State<Base>) -> R<Vec<EquipoVista>> {
    let b = Biblioteca::desde_db(&base.conexion.lock().unwrap()).map_err(txt)?;
    let mut v: Vec<EquipoVista> = b
        .catalogo
        .values()
        .map(|e| EquipoVista {
            codigo: e.codigo.clone(),
            familia: e.familia.clone(),
            descripcion: e.descripcion.clone(),
            unidad: e.unidad.clone(),
            modo_defecto: e.modo_defecto.clone(),
            partida_subcontrata: e.partida_subcontrata.clone(),
            tiempos: b
                .tiempos_de(&e.codigo)
                .filter_map(|t| {
                    b.operaciones.get(&t.operacion_id).map(|o| TiempoVista {
                        operacion_id: o.id.clone(),
                        texto: o.texto.clone(),
                        periodicidad: o.periodicidad.clone(),
                        referencia: o.referencia.clone(),
                        fuente: o.fuente.clone(),
                        verificado: o.verificado.clone(),
                        minutos: t.minutos,
                        tecnicos: t.tecnicos,
                        estado: t.estado.clone(),
                    })
                })
                .collect(),
        })
        .collect();
    v.sort_by(|a, b| (a.familia.as_str(), a.codigo.as_str()).cmp(&(b.familia.as_str(), b.codigo.as_str())));
    Ok(v)
}

#[tauri::command]
fn fijar_tiempo(base: State<Base>, equipo: String, operacion: String, minutos: f64, tecnicos: u32) -> R<()> {
    db::fijar_tiempo(&base.conexion.lock().unwrap(), &equipo, &operacion, minutos, tecnicos).map_err(txt)
}

// ───────────────────────────── Parámetros y tarifa ─────────────────────────────

#[tauri::command]
fn parametros(base: State<Base>, anio: u32) -> R<Vec<(String, f64)>> {
    db::parametros_de(&base.conexion.lock().unwrap(), anio).map_err(txt)
}

#[tauri::command]
fn fijar_parametro(base: State<Base>, anio: u32, nombre: String, valor: f64) -> R<()> {
    db::fijar_parametro(&base.conexion.lock().unwrap(), anio, &nombre, valor).map_err(txt)
}

#[tauri::command]
fn calcular_tarifa(base: State<Base>, anio: u32) -> R<mantenix_core::Tarifa> {
    tarifa(
        &Biblioteca::desde_db(&base.conexion.lock().unwrap()).map_err(txt)?,
        anio,
    )
    .map_err(txt)
}

#[tauri::command]
fn pdf_tarifa(base: State<Base>, anio: u32, ruta: String) -> R<()> {
    let c = base.conexion.lock().unwrap();
    let t = tarifa(&Biblioteca::desde_db(&c).map_err(txt)?, anio).map_err(txt)?;
    let pdf = mantenix_informes::tarifa(&t, &db::empresa(&c).map_err(txt)?).map_err(txt)?;
    std::fs::write(ruta, pdf).map_err(txt)
}

// ───────────────────────────── Trazabilidad ─────────────────────────────

#[tauri::command]
fn historial(base: State<Base>, tabla: Option<String>, limite: u32) -> R<Vec<db::Cambio>> {
    db::historial(&base.conexion.lock().unwrap(), tabla.as_deref(), limite).map_err(txt)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Base por defecto en la carpeta de datos del usuario; se crea con datos de ejemplo.
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let ruta = dir.join("mantenix.db");
            let s = carpeta_semilla(app.handle());
            let c = abrir_o_crear(&ruta, &s.join("normativa"), &s.join("ejemplo")).map_err(|e| e.to_string())?;
            app.manage(Base {
                conexion: Mutex::new(c),
                ruta: Mutex::new(ruta),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            estado,
            fijar_usuario,
            abrir_base,
            crear_base,
            listar_contratos,
            cargar_contrato,
            guardar_contrato,
            borrar_contrato,
            valorar_contrato,
            pdf_hoja_costes,
            catalogo,
            fijar_tiempo,
            parametros,
            fijar_parametro,
            calcular_tarifa,
            pdf_tarifa,
            historial,
        ])
        .run(tauri::generate_context!())
        .expect("error al iniciar Mantenix");
}
