//! Datos de entrada de una valoración: el inventario que se recoge en la visita.

use serde::{Deserialize, Serialize};

/// Contrato a valorar: inventario y condiciones.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contrato {
    /// Nombre único del contrato.
    pub nombre: String,
    /// Año de los parámetros y tarifas aplicados.
    pub anio: u32,
    /// `viviendas` u `otros` (columna de la tabla 3.1 del RITE).
    #[serde(default = "uso_por_defecto")]
    pub uso: String,
    /// Visitas al año contratadas. Si falta, se toma el mínimo que exige la normativa.
    pub visitas_contratadas: Option<u32>,
    #[serde(default)]
    /// Desplazamiento.
    pub desplazamiento: Desplazamiento,
    /// Instalaciones del contrato (RITE, PCI…).
    pub instalaciones: Vec<Instalacion>,
    /// Sustituye al margen del año si se indica.
    pub margen_pct: Option<f64>,
    /// Precio actual del contrato, para comparar (calibración / renovación).
    pub precio_actual_anual: Option<f64>,
    /// Atención de avisos de avería (bolsa incluida en la cuota, todos incluidos o aparte).
    #[serde(default)]
    pub avisos: Avisos,
    /// Datos del documento: cliente, edificio, referencia, firmantes y condiciones (JSON con la
    /// forma que espera [`crate::documento::datos_contrato`]). Vacío si aún no se han rellenado.
    #[serde(default)]
    pub partes: serde_json::Value,
}

/// Cómo se atienden los avisos de avería y con qué datos se estiman.
///
/// Ver [`crate::avisos`] para las tasas de avisos por equipo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Avisos {
    /// `bolsa` (N avisos incluidos; el exceso por tarifa), `incluidos` (todos) o `aparte`.
    #[serde(default = "bolsa")]
    pub modalidad: String,
    /// Avisos incluidos en la bolsa. Si falta, los estimados redondeados hacia arriba.
    #[serde(default)]
    pub bolsa: Option<u32>,
    /// Antigüedad de la instalación: `nueva` (< 5 años), `media` o `antigua` (> 15 años).
    #[serde(default = "media")]
    pub antiguedad: String,
    /// Servicio 24 horas (hotel, residencia, hospital…): más avisos.
    #[serde(default)]
    pub servicio_24h: bool,
    /// Horas de oficial en el edificio por aviso.
    #[serde(default = "hora_y_media")]
    pub horas_por_aviso: f64,
}

impl Default for Avisos {
    fn default() -> Self {
        Avisos {
            modalidad: bolsa(),
            bolsa: None,
            antiguedad: media(),
            servicio_24h: false,
            horas_por_aviso: hora_y_media(),
        }
    }
}

fn bolsa() -> String {
    "bolsa".into()
}
fn media() -> String {
    "media".into()
}
fn hora_y_media() -> f64 {
    1.5
}

fn uso_por_defecto() -> String {
    "otros".into()
}

/// Desplazamiento desde la base hasta el edificio.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Desplazamiento {
    #[serde(default)]
    /// Kilómetros de ida desde la base.
    pub km_ida: f64,
    #[serde(default)]
    /// Minutos de ida desde la base.
    pub minutos_ida: f64,
}

/// Instalación (RITE, PCI…) con sus equipos.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instalacion {
    /// `RITE` o `PCI`.
    pub familia: String,
    #[serde(default)]
    /// Título para los documentos.
    pub titulo: String,
    /// Potencia térmica nominal (solo RITE).
    pub potencia_kw: Option<f64>,
    /// Meses de uso al año (convierte la periodicidad «m» del RITE en visitas).
    #[serde(default = "doce")]
    pub temporada_meses: f64,
    /// Equipos inventariados.
    pub equipos: Vec<LineaEquipo>,
}

fn doce() -> f64 {
    12.0
}

/// Línea del inventario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineaEquipo {
    /// Código del tipo de equipo en el catálogo.
    pub codigo: String,
    /// Número de unidades.
    pub cantidad: f64,
    /// Descripción concreta (marca y modelo); si falta, la del catálogo.
    #[serde(default)]
    pub descripcion: String,
    /// `propio` o `subcontrata`; si falta, el del catálogo.
    pub modo: Option<String>,
}
