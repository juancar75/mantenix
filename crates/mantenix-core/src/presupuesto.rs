//! Presupuestos pequeños: reparaciones (correctivo) y pequeñas obras o mejoras.
//!
//! Se emiten como **carta presupuesto**: se explica al cliente qué equipo se ha roto o
//! desgastado, qué consecuencias tiene y qué repuesto o trabajo se propone, con el importe
//! desglosado y un espacio para su aceptación.
//!
//! Un presupuesto es una lista de líneas con precio de venta. Cada línea puede llevar además
//! su coste, para conocer el margen interno; el coste **nunca** aparece en el PDF del cliente.
//!
//! | Tipo de línea | Precio por defecto | Coste por defecto |
//! |---|---|---|
//! | `hora_oficial` | parámetro `precio_hora_oficial` | `coste_hora_oficial` |
//! | `hora_pareja` | parámetro `precio_hora_pareja` | `coste_hora_oficial` + `coste_hora_ayudante` |
//! | `salida` | parámetro `precio_salida` | `coste_salida` (si existe) |
//! | `material` | coste × (1 + `recargo_material_pct`) | el que se escriba |
//! | `partida` | el que se escriba | el que se escriba |
//!
//! Los valores por defecto los propone [`precios_por_defecto`]; el usuario puede cambiarlos
//! en cada línea. El cálculo de importes y totales está en [`calcular`].

use serde::{Deserialize, Serialize};

use crate::datos::Biblioteca;

/// Tipos de línea admitidos.
pub const TIPOS_LINEA: &[&str] = &["hora_oficial", "hora_pareja", "salida", "material", "partida"];

/// Estados de un presupuesto.
pub const ESTADOS: &[&str] = &["borrador", "enviado", "aceptado", "rechazado"];

/// Línea de un presupuesto.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LineaPresupuesto {
    /// Uno de [`TIPOS_LINEA`].
    pub tipo: String,
    /// Texto que verá el cliente.
    pub descripcion: String,
    /// Unidad (h, ud, m, salida…).
    #[serde(default = "ud")]
    pub unidad: String,
    /// Cantidad.
    pub cantidad: f64,
    /// Precio de venta unitario, sin IVA (€).
    pub precio_unitario: f64,
    /// Coste unitario para la empresa (€). Solo para el margen interno.
    #[serde(default)]
    pub coste_unitario: Option<f64>,
}

fn ud() -> String {
    "ud".into()
}
fn iva_defecto() -> f64 {
    21.0
}
fn validez_defecto() -> u32 {
    30
}
fn estado_defecto() -> String {
    "borrador".into()
}
fn tipo_defecto() -> String {
    "correctivo".into()
}

/// Presupuesto de reparación u obra pequeña.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Presupuesto {
    /// Número visible (p. ej. `P-2026-007`). Vacío al crear: lo asigna [`crate::db::guardar_presupuesto`].
    #[serde(default)]
    pub numero: String,
    /// Fecha de emisión `AAAA-MM-DD`.
    pub fecha: String,
    /// `correctivo` (reparación) u `obra` (pequeña obra o mejora).
    #[serde(default = "tipo_defecto")]
    pub tipo: String,
    /// Cliente (razón social o nombre).
    pub cliente: String,
    /// CIF/NIF del cliente.
    #[serde(default)]
    pub cif: String,
    /// Dirección de la instalación.
    #[serde(default)]
    pub direccion: String,
    /// Persona de contacto.
    #[serde(default)]
    pub contacto: String,
    /// Contrato de mantenimiento relacionado, si lo hay.
    #[serde(default)]
    pub contrato: String,
    /// Título del trabajo.
    pub titulo: String,
    /// Equipo o elemento afectado (p. ej. «Bomba circuladora del primario»).
    #[serde(default)]
    pub equipo: String,
    /// Cuándo y cómo se detectó (p. ej. «revisión trimestral del 14/10/2026»).
    #[serde(default)]
    pub deteccion: String,
    /// Qué se ha roto o desgastado.
    #[serde(default)]
    pub averia: String,
    /// Consecuencias o riesgo de no repararlo.
    #[serde(default)]
    pub consecuencias: String,
    /// Repuesto o trabajo que se propone.
    #[serde(default)]
    pub propuesta: String,
    /// Líneas.
    #[serde(default)]
    pub lineas: Vec<LineaPresupuesto>,
    /// Descuento sobre el subtotal (%).
    #[serde(default)]
    pub descuento_pct: f64,
    /// IVA (%).
    #[serde(default = "iva_defecto")]
    pub iva_pct: f64,
    /// Días de validez de la oferta.
    #[serde(default = "validez_defecto")]
    pub validez_dias: u32,
    /// Condiciones (plazo, forma de pago, exclusiones…). Vacío = las de la empresa.
    #[serde(default)]
    pub condiciones: String,
    /// Uno de [`ESTADOS`].
    #[serde(default = "estado_defecto")]
    pub estado: String,
    /// Notas internas (no salen en el PDF).
    #[serde(default)]
    pub notas_internas: String,
}

impl Presupuesto {
    /// Año del presupuesto (de la fecha), para numerar y elegir parámetros.
    pub fn anio(&self) -> u32 {
        self.fecha.get(..4).and_then(|a| a.parse().ok()).unwrap_or(2026)
    }
}

/// Importe de una línea.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ImporteLinea {
    /// Cantidad × precio unitario (€).
    pub importe: f64,
    /// Cantidad × coste unitario, si se conoce (€).
    pub coste: Option<f64>,
}

/// Totales del presupuesto.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TotalesPresupuesto {
    /// Importe de cada línea, en el mismo orden.
    pub lineas: Vec<ImporteLinea>,
    /// Suma de líneas (€).
    pub subtotal: f64,
    /// Descuento (€).
    pub descuento: f64,
    /// Base imponible (€).
    pub base: f64,
    /// IVA (€).
    pub iva: f64,
    /// Total con IVA (€).
    pub total: f64,
    /// Coste de las líneas con coste conocido (€).
    pub coste: f64,
    /// Líneas sin coste (su margen no se conoce).
    pub lineas_sin_coste: usize,
    /// Base − coste (€).
    pub margen: f64,
    /// Margen sobre la base imponible (%).
    pub margen_pct: f64,
}

fn r2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

/// Calcula importes, IVA, total y margen interno.
pub fn calcular(p: &Presupuesto) -> TotalesPresupuesto {
    let lineas: Vec<ImporteLinea> = p
        .lineas
        .iter()
        .map(|l| ImporteLinea {
            importe: r2(l.cantidad * l.precio_unitario),
            coste: l.coste_unitario.map(|c| r2(l.cantidad * c)),
        })
        .collect();
    let subtotal = r2(lineas.iter().map(|l| l.importe).sum());
    let descuento = r2(subtotal * p.descuento_pct / 100.0);
    let base = r2(subtotal - descuento);
    let iva = r2(base * p.iva_pct / 100.0);
    let coste = r2(lineas.iter().filter_map(|l| l.coste).sum());
    let margen = r2(base - coste);
    TotalesPresupuesto {
        lineas_sin_coste: lineas.iter().filter(|l| l.coste.is_none()).count(),
        lineas,
        subtotal,
        descuento,
        base,
        iva,
        total: r2(base + iva),
        coste,
        margen,
        margen_pct: if base > 0.0 { r2(margen / base * 100.0) } else { 0.0 },
    }
}

/// Precio y coste que se proponen para una línea nueva de un tipo.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PrecioDefecto {
    /// Tipo de línea.
    pub tipo: String,
    /// Descripción propuesta.
    pub descripcion: String,
    /// Unidad.
    pub unidad: String,
    /// Precio de venta propuesto (€), si hay parámetro.
    pub precio_unitario: Option<f64>,
    /// Coste propuesto (€), si hay parámetro.
    pub coste_unitario: Option<f64>,
}

/// Precios por defecto de cada tipo de línea, con los parámetros del año.
/// También devuelve el recargo de material (%), para calcular el precio desde el coste.
pub fn precios_por_defecto(b: &Biblioteca, anio: u32) -> (Vec<PrecioDefecto>, Option<f64>) {
    let p = |n: &str| b.parametro(anio, n).ok();
    let of = p("coste_hora_oficial");
    let ay = p("coste_hora_ayudante");
    let fila = |tipo: &str, desc: &str, unidad: &str, precio, coste| PrecioDefecto {
        tipo: tipo.into(),
        descripcion: desc.into(),
        unidad: unidad.into(),
        precio_unitario: precio,
        coste_unitario: coste,
    };
    (
        vec![
            fila(
                "hora_oficial",
                "Mano de obra oficial",
                "h",
                p("precio_hora_oficial"),
                of,
            ),
            fila(
                "hora_pareja",
                "Mano de obra oficial + ayudante",
                "h",
                p("precio_hora_pareja"),
                of.zip(ay).map(|(a, b)| a + b),
            ),
            fila(
                "salida",
                "Desplazamiento",
                "salida",
                p("precio_salida"),
                p("coste_salida"),
            ),
            fila("material", "", "ud", None, None),
            fila("partida", "", "ud", None, None),
        ],
        p("recargo_material_pct"),
    )
}

/// Precio de venta de un material a partir de su coste y el recargo (% sobre coste).
pub fn precio_material(coste: f64, recargo_pct: f64) -> f64 {
    r2(coste * (1.0 + recargo_pct / 100.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ejemplo() -> Presupuesto {
        serde_json::from_str(
            r#"{"fecha":"2026-10-19","cliente":"Comunidad X","titulo":"Sustituir bomba",
            "lineas":[
              {"tipo":"hora_pareja","descripcion":"Mano de obra","unidad":"h","cantidad":3,"precio_unitario":80,"coste_unitario":47.91},
              {"tipo":"salida","descripcion":"Desplazamiento","cantidad":1,"precio_unitario":30},
              {"tipo":"material","descripcion":"Bomba","cantidad":1,"precio_unitario":650,"coste_unitario":500}
            ],"descuento_pct":5}"#,
        )
        .unwrap()
    }

    #[test]
    fn totales_con_descuento_e_iva() {
        let t = calcular(&ejemplo());
        assert_eq!(t.subtotal, 920.0); // 240 + 30 + 650
        assert_eq!(t.descuento, 46.0);
        assert_eq!(t.base, 874.0);
        assert_eq!(t.iva, 183.54);
        assert_eq!(t.total, 1057.54);
        assert_eq!(t.coste, 643.73); // 143,73 + 500
        assert_eq!(t.lineas_sin_coste, 1);
        assert_eq!(t.margen, 230.27);
    }

    #[test]
    fn valores_por_defecto() {
        let p = ejemplo();
        assert_eq!(p.iva_pct, 21.0);
        assert_eq!(p.validez_dias, 30);
        assert_eq!(p.estado, "borrador");
        assert_eq!(p.anio(), 2026);
        assert_eq!(precio_material(100.0, 30.0), 130.0);
    }
}
