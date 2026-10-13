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
