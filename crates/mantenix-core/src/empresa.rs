//! Datos de la empresa mantenedora en los documentos: membrete por ámbito.
//!
//! Según el ámbito del trabajo (RITE, PCI, baja tensión…) el documento lleva el **logo** y el
//! **número de empresa habilitada** que corresponden. Todo se guarda en el JSON de la tabla
//! `empresa` (con historial de cambios), con esta forma:
//!
//! ```json
//! {
//!   "razon_social": "…", "cif": "…", "domicilio": "…",
//!   "habilitaciones": [["Empresa mantenedora de instalaciones térmicas (RITE)", "00/EMT-0000"], …],
//!   "logo": {"tipo": "png", "datos": "<base64>"},
//!   "ambitos": {
//!     "RITE": {"logo": {"tipo": "png", "datos": "<base64>"}},
//!     "PCI":  {"logo": {"tipo": "svg", "datos": "<base64>"}}
//!   }
//! }
//! ```
//!
//! * El logo de un ámbito sustituye al general; si no hay ninguno, el documento sale sin logo.
//! * Las habilitaciones de un ámbito se eligen por su texto (ver [`AMBITOS`]); si un ámbito no
//!   tiene ninguna que coincida, se muestran todas.

use base64::Engine;
use serde::Serialize;
use serde_json::{json, Value};

use crate::Error;

/// Ámbitos de trabajo: (código, nombre visible, palabras que identifican su habilitación).
pub const AMBITOS: &[(&str, &str, &[&str])] = &[
    (
        "RITE",
        "Instalaciones térmicas (RITE)",
        &["térmic", "termic", "rite", "emt"],
    ),
    (
        "PCI",
        "Protección contra incendios (RIPCI)",
        &["incendio", "ripci", "pci", "emi"],
    ),
    (
        "REBT",
        "Baja tensión (REBT)",
        &["baja tensión", "baja tension", "rebt", "eibt"],
    ),
];

/// Tamaño máximo de un logo (bytes).
pub const LOGO_MAX: usize = 1_000_000;

/// Membrete de un documento.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Membrete {
    /// Habilitaciones que se muestran: (descripción, número).
    pub habilitaciones: Vec<(String, String)>,
    /// Nombre de fichero del logo dentro de la plantilla (`logo.png`…), si lo hay.
    pub logo: Option<String>,
    /// Contenido del logo.
    #[serde(skip)]
    pub logo_datos: Vec<u8>,
}

/// Formato de imagen por su contenido: `png`, `jpg` o `svg`.
pub fn tipo_imagen(datos: &[u8]) -> Option<&'static str> {
    if datos.starts_with(b"\x89PNG") {
        Some("png")
    } else if datos.starts_with(&[0xFF, 0xD8]) {
        Some("jpg")
    } else {
        let inicio = String::from_utf8_lossy(&datos[..datos.len().min(512)]).to_lowercase();
        (inicio.contains("<svg")).then_some("svg")
    }
}

fn leer_logo(v: &Value) -> Option<(String, Vec<u8>)> {
    let tipo = v.get("tipo")?.as_str()?.to_string();
    let datos = base64::engine::general_purpose::STANDARD
        .decode(v.get("datos")?.as_str()?)
        .ok()?;
    Some((tipo, datos))
}

/// Membrete para un ámbito (`""` = general: todas las habilitaciones y el logo general).
pub fn membrete(empresa: &Value, ambito: &str) -> Membrete {
    let todas: Vec<(String, String)> = empresa
        .get("habilitaciones")
        .and_then(Value::as_array)
        .map(|v| {
            v.iter()
                .filter_map(|h| Some((h.get(0)?.as_str()?.to_string(), h.get(1)?.as_str()?.to_string())))
                .collect()
        })
        .unwrap_or_default();
    let claves = AMBITOS.iter().find(|a| a.0 == ambito).map(|a| a.2).unwrap_or(&[]);
    let propias: Vec<_> = todas
        .iter()
        .filter(|(n, num)| {
            let t = format!("{n} {num}").to_lowercase();
            claves.iter().any(|k| t.contains(k))
        })
        .cloned()
        .collect();
    let logo = empresa
        .get("ambitos")
        .and_then(|a| a.get(ambito))
        .and_then(|a| a.get("logo"))
        .and_then(leer_logo)
        .or_else(|| empresa.get("logo").and_then(leer_logo));
    let (logo, logo_datos) = match logo {
        Some((tipo, datos)) => (Some(format!("logo.{tipo}")), datos),
        None => (None, Vec::new()),
    };
    Membrete {
        habilitaciones: if propias.is_empty() { todas } else { propias },
        logo,
        logo_datos,
    }
}

/// Pone (o quita, con `datos = None`) el logo de un ámbito (`""` = logo general).
pub fn fijar_logo(empresa: &mut Value, ambito: &str, datos: Option<&[u8]>) -> Result<(), Error> {
    if !ambito.is_empty() && !AMBITOS.iter().any(|a| a.0 == ambito) {
        return Err(Error::Db(format!("Ámbito no válido: {ambito}")));
    }
    let valor = match datos {
        None => Value::Null,
        Some(d) => {
            if d.len() > LOGO_MAX {
                return Err(Error::Db(
                    "El logo ocupa más de 1 MB; redúzcalo antes de cargarlo.".into(),
                ));
            }
            let tipo = tipo_imagen(d).ok_or_else(|| Error::Db("El logo debe ser PNG, JPG o SVG.".into()))?;
            json!({"tipo": tipo, "datos": base64::engine::general_purpose::STANDARD.encode(d)})
        }
    };
    if !empresa.is_object() {
        *empresa = json!({});
    }
    let o = empresa.as_object_mut().expect("objeto");
    if ambito.is_empty() {
        o.insert("logo".into(), valor);
    } else {
        let ambitos = o.entry("ambitos").or_insert_with(|| json!({}));
        if !ambitos.is_object() {
            *ambitos = json!({});
        }
        let a = ambitos
            .as_object_mut()
            .expect("objeto")
            .entry(ambito)
            .or_insert_with(|| json!({}));
        if !a.is_object() {
            *a = json!({});
        }
        a.as_object_mut().expect("objeto").insert("logo".into(), valor);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const PNG: &[u8] = b"\x89PNG\r\n\x1a\n0000";

    fn empresa() -> Value {
        json!({"razon_social": "X", "habilitaciones": [
            ["Empresa mantenedora de protección contra incendios (RIPCI)", "00/EMI-1"],
            ["Empresa mantenedora de instalaciones térmicas (RITE)", "00/EMT-2"],
            ["Empresa instaladora de baja tensión (REBT)", "00/EIBT-3"]]})
    }

    #[test]
    fn habilitacion_segun_ambito() {
        let e = empresa();
        assert_eq!(
            membrete(&e, "RITE").habilitaciones,
            vec![(
                "Empresa mantenedora de instalaciones térmicas (RITE)".into(),
                "00/EMT-2".into()
            )]
        );
        assert_eq!(membrete(&e, "PCI").habilitaciones[0].1, "00/EMI-1");
        assert_eq!(membrete(&e, "REBT").habilitaciones[0].1, "00/EIBT-3");
        assert_eq!(membrete(&e, "").habilitaciones.len(), 3);
    }

    #[test]
    fn logo_de_ambito_o_general() {
        let mut e = empresa();
        assert_eq!(membrete(&e, "PCI").logo, None);
        fijar_logo(&mut e, "", Some(PNG)).unwrap();
        assert_eq!(membrete(&e, "PCI").logo.as_deref(), Some("logo.png"));
        fijar_logo(&mut e, "PCI", Some(b"<svg xmlns='x'></svg>")).unwrap();
        let m = membrete(&e, "PCI");
        assert_eq!(m.logo.as_deref(), Some("logo.svg"));
        assert!(m.logo_datos.starts_with(b"<svg"));
        assert_eq!(
            membrete(&e, "RITE").logo.as_deref(),
            Some("logo.png"),
            "sin logo propio usa el general"
        );
        fijar_logo(&mut e, "PCI", None).unwrap();
        assert_eq!(membrete(&e, "PCI").logo.as_deref(), Some("logo.png"));
        assert!(fijar_logo(&mut e, "PCI", Some(b"no es imagen")).is_err());
        assert!(fijar_logo(&mut e, "OTRO", Some(PNG)).is_err());
    }
}
