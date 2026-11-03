//! # Mantenix — núcleo de cálculo
//!
//! Biblioteca que contiene toda la lógica de la aplicación, sin interfaz gráfica.
//! La usan la aplicación de escritorio (`app/`) y la línea de comandos (`mantenix-cli`).
//!
//! ## Flujo
//!
//! ```text
//! SQLite ──► Biblioteca (normativa + empresa) ─┐
//!                                              ├─► motor::valorar ─► Valoracion ─► informes PDF
//! SQLite ──► Contrato (inventario) ────────────┘
//! ```
//!
//! ## Módulos
//!
//! - [`db`]: base de datos SQLite (fuente única de datos; los CSV solo son semilla).
//! - [`datos`]: estructuras de la biblioteca normativa (RITE, RIPCI) y de los datos de empresa.
//! - [`modelo`]: inventario de un contrato (entrada).
//! - [`motor`]: valoración: horas, costes y precio de venta.
//! - [`avisos`]: estimación de avisos de avería al año.
//! - [`documento`]: datos del documento de contrato desde el inventario y la valoración.
//! - [`tarifa`]: tarifa propia por equipo y revisión.
//! - [`presupuesto`]: presupuestos de reparación y pequeñas obras.
//! - [`empresa`]: membrete de los documentos (logo y habilitación según el ámbito).

#![warn(missing_docs)]

pub mod avisos;
pub mod datos;
pub mod db;
pub mod documento;
pub mod empresa;
pub mod modelo;
pub mod motor;
pub mod presupuesto;
pub mod tarifa;

pub use datos::Biblioteca;
pub use modelo::Contrato;
pub use motor::{valorar, Valoracion};
pub use presupuesto::Presupuesto;
pub use tarifa::{tarifa, Tarifa};

/// Errores del núcleo. Los mensajes están en español porque se muestran al usuario.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// No se puede abrir un fichero de semilla.
    #[error("No se puede abrir {0}: {1}")]
    Fichero(String, String),
    /// Error de formato en un CSV de semilla.
    #[error("Error leyendo {0}: {1}")]
    Csv(String, String),
    /// Falta un parámetro (coste hora, margen…) para el año y los anteriores.
    #[error("Falta el parámetro «{0}» para el año {1}")]
    Parametro(String, u32),
    /// Margen igual o superior al 100 %.
    #[error("Margen no válido: {0} %")]
    Margen(f64),
    /// Error de SQLite.
    #[error("Base de datos: {0}")]
    Db(String),
    /// Contrato o presupuesto inexistente.
    #[error("No existe «{0}»")]
    NoEncontrado(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn raiz() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
    }
    fn normativa() -> PathBuf {
        raiz().join("datos/semilla/normativa")
    }
    fn ejemplo() -> PathBuf {
        raiz().join("datos/semilla/ejemplo")
    }
    fn biblioteca() -> Biblioteca {
        Biblioteca::desde_csv(&normativa(), &ejemplo()).unwrap()
    }

    #[test]
    fn carga_biblioteca() {
        let b = biblioteca();
        assert!(b.catalogo.len() > 50);
        assert_eq!(b.operaciones.values().filter(|o| o.id.starts_with("RIPCI")).count(), 97);
        assert_eq!(
            b.operaciones.values().filter(|o| o.id.starts_with("RITE-T33")).count(),
            43
        );
        assert!(
            (b.parametro(2027, "margen_pct").unwrap() - 25.0).abs() < 1e-9,
            "hereda el año anterior"
        );
    }

    #[test]
    fn periodicidades() {
        assert_eq!(motor::veces_anio("m", 7.0, 4), 7.0);
        assert_eq!(motor::veces_anio("3m", 12.0, 4), 4.0);
        assert_eq!(motor::veces_anio("5a", 12.0, 4), 0.0);
        assert_eq!(motor::veces_anio("S*", 12.0, 4), 0.0);
    }

    #[test]
    fn precio_margen_sobre_venta() {
        let b = biblioteca();
        let c: Contrato = serde_json::from_str(
            r#"{
            "nombre":"prueba","anio":2026,"visitas_contratadas":1,
            "instalaciones":[{"familia":"RITE","potencia_kw":10,"equipos":[{"codigo":"SPLIT-INT","cantidad":1}]}]}"#,
        )
        .unwrap();
        let v = valorar(&b, &c).unwrap();
        let t = &v.totales;
        assert!((t.precio_anual - t.coste_total / 0.75).abs() < 0.02);
        // Split interior ≤12 kW, otros usos: tabla 3.1 = cada 2 años → evaporador (t) 0,5 veces/año;
        // filtros (m) y revisión (2t) limitados a la única visita contratada: 10 + 20×0,5 + 15 = 35 min
        let esperado_mo = 35.0 / 60.0 * 28.72;
        assert!((t.mano_obra - esperado_mo).abs() < 0.02, "{}", t.mano_obra);
    }

    #[test]
    fn pci_subcontrata_minimo_visita() {
        let b = biblioteca();
        let c: Contrato = serde_json::from_str(
            r#"{
            "nombre":"prueba","anio":2026,
            "instalaciones":[{"familia":"PCI","equipos":[{"codigo":"EXT-POLVO","cantidad":2}]}]}"#,
        )
        .unwrap();
        let v = valorar(&b, &c).unwrap();
        assert_eq!(v.visitas, 4);
        assert_eq!(v.subcontrata_visita_suma, 10.0); // 2 × 5 € (tarifa de ejemplo)
        assert_eq!(v.subcontrata_visita_facturada, 90.0);
        assert_eq!(v.totales.subcontrata, 360.0);
    }

    #[test]
    fn sqlite_ida_y_vuelta() {
        let mut c = db::abrir(std::path::Path::new(":memory:")).unwrap();
        assert!(db::inicializar(&mut c, &normativa(), &ejemplo()).unwrap());
        assert!(
            !db::inicializar(&mut c, &normativa(), &ejemplo()).unwrap(),
            "la semilla solo se carga una vez"
        );
        let b_db = Biblioteca::desde_db(&c).unwrap();
        let b_csv = biblioteca();
        assert_eq!(b_db.operaciones.len(), b_csv.operaciones.len());
        assert_eq!(b_db.tiempos.len(), b_csv.tiempos.len());

        let k: Contrato = serde_json::from_str(
            r#"{
            "nombre":"rt","anio":2026,"visitas_contratadas":4,"desplazamiento":{"km_ida":10,"minutos_ida":20},
            "instalaciones":[
              {"familia":"RITE","titulo":"Clima","potencia_kw":30,"equipos":[{"codigo":"VRF-INT","cantidad":7}]},
              {"familia":"PCI","titulo":"PCI","equipos":[{"codigo":"EXT-POLVO","cantidad":10}]}]}"#,
        )
        .unwrap();
        let id = db::guardar_contrato(&mut c, &k).unwrap();
        let (id2, k2) = db::cargar_contrato(&c, "rt").unwrap();
        assert_eq!(id, id2);
        let v1 = valorar(&b_csv, &k).unwrap();
        let v2 = valorar(&b_db, &k2).unwrap();
        assert_eq!(v1.totales.precio_anual, v2.totales.precio_anual);
        db::guardar_valoracion(&c, &id, &v2).unwrap();
        // volver a guardar con el mismo nombre sustituye el inventario
        db::guardar_contrato(&mut c, &k).unwrap();
        assert_eq!(db::listar_contratos(&c).unwrap().len(), 1);
    }

    #[test]
    fn auditoria_registra_cambios_con_usuario() {
        let mut c = db::abrir(std::path::Path::new(":memory:")).unwrap();
        db::inicializar(&mut c, &normativa(), &ejemplo()).unwrap();
        db::fijar_usuario(&c, "juancar").unwrap();
        db::fijar_parametro(&c, 2026, "margen_pct", 28.0).unwrap();
        let h = db::historial(&c, Some("parametro"), 1).unwrap();
        assert_eq!(h[0].accion, "cambio");
        assert_eq!(h[0].usuario.as_deref(), Some("juancar"));
        assert!(h[0].antes.as_ref().unwrap().contains("25"));
        assert!(h[0].despues.as_ref().unwrap().contains("28"));
        // la carga inicial queda atribuida a «semilla»
        let todo = db::historial(&c, Some("operacion"), 1000).unwrap();
        assert!(todo.iter().all(|x| x.usuario.as_deref() == Some("semilla")));
    }

    #[test]
    fn presupuestos_numeracion_y_guardado() {
        let c = db::abrir(std::path::Path::new(":memory:")).unwrap();
        db::fijar_usuario(&c, "juancar").unwrap();
        let mut p: Presupuesto = serde_json::from_str(
            r#"{"fecha":"2026-10-19","cliente":"Comunidad X","titulo":"Cambio de bomba",
            "lineas":[{"tipo":"material","descripcion":"Bomba","cantidad":1,"precio_unitario":650,"coste_unitario":500}]}"#,
        )
        .unwrap();
        assert_eq!(db::guardar_presupuesto(&c, &p).unwrap(), "P-2026-001");
        assert_eq!(db::guardar_presupuesto(&c, &p).unwrap(), "P-2026-002");
        // guardar con número existente sustituye
        p.numero = "P-2026-001".into();
        p.estado = "enviado".into();
        db::guardar_presupuesto(&c, &p).unwrap();
        let l = db::listar_presupuestos(&c).unwrap();
        assert_eq!(l.len(), 2);
        assert_eq!(db::cargar_presupuesto(&c, "P-2026-001").unwrap().estado, "enviado");
        assert_eq!(l[0].total, 786.5);
        assert_eq!(l[0].instalacion, "", "sin contrato ni dirección");
        p.contrato = "Mantenimiento RITE Edificio A".into();
        p.numero = "P-2026-002".into();
        db::guardar_presupuesto(&c, &p).unwrap();
        let l = db::listar_presupuestos(&c).unwrap();
        assert!(l.iter().any(|r| r.instalacion == "Mantenimiento RITE Edificio A"));
        let h = db::historial(&c, Some("presupuesto"), 10).unwrap();
        assert_eq!(h.len(), 4);
        assert_eq!(h[0].usuario.as_deref(), Some("juancar"));
        // otro año empieza en 001
        p.numero.clear();
        p.fecha = "2027-01-02".into();
        assert_eq!(db::guardar_presupuesto(&c, &p).unwrap(), "P-2027-001");
        db::borrar_presupuesto(&c, "P-2027-001").unwrap();
        assert!(db::cargar_presupuesto(&c, "P-2027-001").is_err());
        // validaciones
        p.cliente.clear();
        assert!(db::guardar_presupuesto(&c, &p).is_err());
    }

    #[test]
    fn migra_bases_antiguas() {
        let ruta = std::env::temp_dir().join(format!("mantenix-migra-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&ruta);
        {
            let c = rusqlite::Connection::open(&ruta).unwrap();
            c.execute_batch(
                "CREATE TABLE contrato (id TEXT PRIMARY KEY, cliente_id TEXT, nombre TEXT NOT NULL UNIQUE,
                 anio INTEGER NOT NULL, uso TEXT NOT NULL DEFAULT 'otros', visitas_contratadas INTEGER,
                 km_ida REAL NOT NULL DEFAULT 0, minutos_ida REAL NOT NULL DEFAULT 0, margen_pct REAL,
                 precio_actual_anual REAL, actualizado TEXT NOT NULL DEFAULT (datetime('now')));",
            )
            .unwrap();
        }
        let mut c = db::abrir(&ruta).unwrap();
        let mut k: Contrato = serde_json::from_str(r#"{"nombre":"m","anio":2026,"instalaciones":[]}"#).unwrap();
        k.partes = serde_json::json!({"cliente": {"razon_social": "X"}});
        db::guardar_contrato(&mut c, &k).unwrap();
        let (_, k2) = db::cargar_contrato(&c, "m").unwrap();
        assert_eq!(k2.partes["cliente"]["razon_social"], "X");
        assert_eq!(k2.avisos.modalidad, "bolsa");
        drop(c);
        let _ = std::fs::remove_file(&ruta);
    }

    #[test]
    fn avisos_bolsa_incluidos_y_aparte() {
        let b = biblioteca();
        let mut c: Contrato = serde_json::from_str(
            r#"{"nombre":"av","anio":2026,"uso":"viviendas","desplazamiento":{"km_ida":20,"minutos_ida":25},
            "instalaciones":[{"familia":"RITE","potencia_kw":300,"equipos":[
              {"codigo":"CALD-GAS","cantidad":2},{"codigo":"BOMBA","cantidad":4}]}]}"#,
        )
        .unwrap();
        // (2 × 1,2 + 4 × 0,1) × 1,0 (media) × 1,2 (viviendas) = 3,36 → bolsa de 4
        let v = valorar(&b, &c).unwrap();
        assert_eq!(v.avisos_averia.estimados, 3.36);
        assert_eq!(v.avisos_averia.incluidos, 4);
        let coste_aviso = 1.5 * 28.72 + 50.0 / 60.0 * 28.72 + 40.0 * 0.15;
        assert!((v.avisos_averia.coste_aviso - coste_aviso).abs() < 0.01);
        assert!((v.totales.avisos - 4.0 * coste_aviso).abs() < 0.05);

        c.avisos.modalidad = "aparte".into();
        let v2 = valorar(&b, &c).unwrap();
        assert_eq!(v2.totales.avisos, 0.0);
        assert!(v2.totales.precio_anual < v.totales.precio_anual);

        c.avisos.modalidad = "bolsa".into();
        c.avisos.bolsa = Some(2);
        c.avisos.antiguedad = "antigua".into();
        let v3 = valorar(&b, &c).unwrap();
        assert_eq!(v3.avisos_averia.incluidos, 2);
        assert!(v3.avisos.iter().any(|a| a.contains("bolsa")));
    }

    #[test]
    fn modelos_semilla_y_adaptacion() {
        let c = db::abrir(std::path::Path::new(":memory:")).unwrap();
        let fichero = raiz().join("datos/semilla/presupuestos/modelos.json");
        let n = db::cargar_modelos_semilla(&c, &fichero).unwrap();
        assert!(n >= 20, "{n} modelos");
        assert_eq!(
            db::cargar_modelos_semilla(&c, &fichero).unwrap(),
            0,
            "solo si no hay modelos"
        );
        let modelos = db::listar_modelos(&c).unwrap();
        assert!(modelos.iter().all(|m| m.origen == "semilla" && !m.lineas.is_empty()));

        // Presupuesto desde el modelo: precios del año (mano de obra) y coste + recargo (material)
        let b = biblioteca();
        let m = modelos.iter().find(|m| m.codigo == "UTA-CORREAS").unwrap();
        let p = presupuesto::desde_modelo(m, &b, "2026-11-02");
        let hora = p.lineas.iter().find(|l| l.tipo == "hora_oficial").unwrap();
        assert_eq!(hora.precio_unitario, b.parametro(2026, "precio_hora_oficial").unwrap());
        let correa = p.lineas.iter().find(|l| l.tipo == "material").unwrap();
        let r = b.parametro(2026, "recargo_material_pct").unwrap();
        assert_eq!(correa.precio_unitario, presupuesto::precio_material(18.0, r));
        assert!(presupuesto::calcular(&p).total > 0.0);

        // Adaptar: guardar como modelo propio con otro código
        let mut q = p.clone();
        q.equipo = "el extractor de la cocina".into();
        let propio = presupuesto::a_modelo(&q, "ext-correas", "Extractor / ventilador", "Correas de extractor");
        assert_eq!(propio.codigo, "EXT-CORREAS");
        assert_eq!(propio.origen, "propio");
        assert!(propio
            .lineas
            .iter()
            .filter(|l| l.tipo == "hora_oficial")
            .all(|l| l.precio_unitario == 0.0));
        db::guardar_modelo(&c, &propio).unwrap();
        assert_eq!(db::listar_modelos(&c).unwrap().len(), n + 1);
        db::borrar_modelo(&c, "EXT-CORREAS").unwrap();
    }

    #[test]
    fn presupuestos_precios_por_defecto() {
        let b = biblioteca();
        let (filas, recargo) = presupuesto::precios_por_defecto(&b, 2026);
        let pareja = filas.iter().find(|f| f.tipo == "hora_pareja").unwrap();
        assert!((pareja.coste_unitario.unwrap() - (28.72 + 19.19)).abs() < 1e-9);
        assert!(
            pareja.precio_unitario.is_some(),
            "la semilla de ejemplo trae precios de venta"
        );
        assert!(recargo.is_some());
    }
}
