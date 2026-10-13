//! Tarifa propia: precio por unidad y revisión, calculado desde los costes del año.

use serde::Serialize;

use crate::datos::Biblioteca;
use crate::motor::veces_anio;
use crate::Error;

/// Partida de la tarifa propia.
#[derive(Debug, Clone, Serialize)]
pub struct FilaTarifa {
    /// Código del tipo de equipo en el catálogo.
    pub codigo: String,
    /// `RITE`, `PCI` u otra familia sin régimen normativo propio.
    pub familia: String,
    /// Descripción (marca y modelo si se conoce).
    pub descripcion: String,
    /// Unidad de medición.
    pub unidad: String,
    /// `medios propios` o subcontratista.
    pub origen: String,
    /// Minutos de una revisión completa (todas las operaciones una vez).
    pub minutos_revision: f64,
    /// Coste de una revisión completa por unidad (€).
    pub coste_revision: f64,
    /// Precio de venta de una revisión por unidad (€).
    pub precio_revision: f64,
    /// Coste anual con las periodicidades de la tabla 3.3 / anexo II (12 meses de uso).
    pub coste_anual_regimen: f64,
    /// Precio anual con las periodicidades reglamentarias completas (€).
    pub precio_anual_regimen: f64,
}

/// Tarifa propia de un año.
#[derive(Debug, Clone, Serialize)]
pub struct Tarifa {
    /// Año de los parámetros y tarifas aplicados.
    pub anio: u32,
    /// Coste empresa por hora del oficial (€/h).
    pub coste_hora_oficial: f64,
    /// Coste empresa por hora del ayudante (€/h).
    pub coste_hora_ayudante: f64,
    /// Indirectos + contingencia (%).
    pub recargo_pct: f64,
    /// Margen sobre el precio de venta (%).
    pub margen_pct: f64,
    /// Partidas de la tarifa.
    pub filas: Vec<FilaTarifa>,
}

fn r2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

/// Calcula la tarifa propia del año a partir de tiempos, costes y tarifas de subcontrata.
pub fn tarifa(b: &Biblioteca, anio: u32) -> Result<Tarifa, Error> {
    let of = b.parametro(anio, "coste_hora_oficial")?;
    let ay = b.parametro(anio, "coste_hora_ayudante")?;
    let recargo = b.parametro(anio, "indirectos_pct")? + b.parametro(anio, "contingencia_pct")?;
    let margen = b.parametro(anio, "margen_pct")?;
    let a_precio = |coste: f64| coste * (1.0 + recargo / 100.0) / (1.0 - margen / 100.0);

    let mut filas = Vec::new();
    let mut codigos: Vec<_> = b.catalogo.keys().cloned().collect();
    codigos.sort();
    for cod in codigos {
        let eq = &b.catalogo[&cod];
        let tiempos: Vec<_> = b
            .tiempos_de(&cod)
            .filter(|t| {
                b.operaciones
                    .get(&t.operacion_id)
                    .is_some_and(|o| o.tipo == "preventivo")
            })
            .collect();
        if !tiempos.is_empty() {
            let (mut min, mut cr, mut ca) = (0.0, 0.0, 0.0);
            for t in &tiempos {
                let op = &b.operaciones[&t.operacion_id];
                let hora = of + t.tecnicos.saturating_sub(1) as f64 * ay;
                let c1 = t.minutos / 60.0 * hora + t.consumibles_eur;
                let veces_rev = if op.periodicidad == "S*" { 0.0 } else { 1.0 };
                min += t.minutos * t.tecnicos as f64 * veces_rev;
                cr += c1 * veces_rev;
                ca += c1 * veces_anio(&op.periodicidad, 12.0, 4);
            }
            filas.push(FilaTarifa {
                codigo: cod.clone(),
                familia: eq.familia.clone(),
                descripcion: eq.descripcion.clone(),
                unidad: eq.unidad.clone(),
                origen: "medios propios".into(),
                minutos_revision: min,
                coste_revision: r2(cr),
                precio_revision: r2(a_precio(cr)),
                coste_anual_regimen: r2(ca),
                precio_anual_regimen: r2(a_precio(ca)),
            });
        } else if let Some(pt) = b.partida(&eq.partida_subcontrata, anio) {
            if let (Some(pr), "preventivo") = (pt.precio, pt.tipo.as_str()) {
                filas.push(FilaTarifa {
                    codigo: cod.clone(),
                    familia: eq.familia.clone(),
                    descripcion: eq.descripcion.clone(),
                    unidad: pt.unidad.clone(),
                    origen: format!("subcontrata {}", pt.proveedor),
                    minutos_revision: 0.0,
                    coste_revision: pr,
                    precio_revision: r2(a_precio(pr)),
                    coste_anual_regimen: r2(pr * 4.0),
                    precio_anual_regimen: r2(a_precio(pr * 4.0)),
                });
            }
        }
    }
    Ok(Tarifa {
        anio,
        coste_hora_oficial: of,
        coste_hora_ayudante: ay,
        recargo_pct: recargo,
        margen_pct: margen,
        filas,
    })
}
