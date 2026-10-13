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
//! - [`tarifa`]: tarifa propia por equipo y revisión.

#![warn(missing_docs)]

pub mod datos;
pub mod db;
pub mod modelo;
pub mod motor;
pub mod tarifa;

pub use datos::Biblioteca;
pub use modelo::Contrato;
pub use motor::{valorar, Valoracion};
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
    /// Contrato inexistente.
    #[error("No existe el contrato «{0}»")]
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
}
