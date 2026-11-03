//! Estimación de avisos de avería al año.
//!
//! Avisos/año = Σ (unidades × tasa del tipo de equipo) × factor de antigüedad × factor de uso.
//!
//! Las tasas son **estimaciones de partida** sin datos históricos: hay que ajustarlas con los
//! avisos reales de los contratos (ver `docs/metodo-calculo.md`).

/// Avisos al año por unidad, según el código del catálogo. Si un código no está, se usa
/// [`tasa_por_defecto`].
pub const TASAS: &[(&str, f64)] = &[
    ("CALD-GAS", 1.2),
    ("CALD-GASOLEO", 1.5),
    ("CALD-BIO", 1.5),
    ("CALD-MURAL", 0.3),
    ("CALENT-GAS", 0.3),
    ("BC-ACS", 0.3),
    ("ENFRIADORA", 1.0),
    ("ROOFTOP", 0.8),
    ("TORRE", 0.5),
    ("UTA", 0.5),
    ("RECUP", 0.2),
    ("VRF-EXT", 0.5),
    ("VRF-INT", 0.15),
    ("VRF-BS", 0.05),
    ("SPLIT-EXT", 0.15),
    ("SPLIT-INT", 0.15),
    ("FANCOIL", 0.1),
    ("BOMBA", 0.1),
    ("VENT", 0.2),
    ("VASO-EXP", 0.05),
    ("TRAT-AGUA", 0.2),
    ("ACS-SIS", 0.3),
    ("SOLAR-CAMPO", 0.3),
    ("CONTROL", 0.3),
    ("DET-CENTRAL-ANA", 0.5),
    ("DET-CENTRAL-CONV", 0.5),
    ("DET-OPT", 0.02),
    ("DET-TERM", 0.02),
    ("DET-MULTI", 0.02),
    ("GP-S", 0.4),
    ("GP-M", 0.5),
    ("GP-L", 0.7),
    ("SCTEH-SIS", 0.3),
    ("SCTEH-VENT", 0.2),
    ("CO-CENTRAL", 0.3),
    ("CO-DET", 0.03),
];

/// Tasa de un equipo sin entrada en [`TASAS`] (difusores, extintores, señales…).
pub fn tasa_por_defecto(familia: &str) -> f64 {
    if familia == "PCI" {
        0.0
    } else {
        0.02
    }
}

/// Tasa de avisos al año por unidad de un código de catálogo.
pub fn tasa(codigo: &str, familia: &str) -> f64 {
    TASAS
        .iter()
        .find(|(c, _)| *c == codigo)
        .map(|(_, t)| *t)
        .unwrap_or_else(|| tasa_por_defecto(familia))
}

/// Factor por antigüedad de la instalación.
pub fn factor_antiguedad(antiguedad: &str) -> f64 {
    match antiguedad {
        "nueva" => 0.7,
        "antigua" => 1.5,
        _ => 1.0,
    }
}

/// Factor por uso: las comunidades de vecinos avisan más; el servicio 24 h, más aún.
pub fn factor_uso(viviendas: bool, servicio_24h: bool) -> f64 {
    if servicio_24h {
        1.5
    } else if viviendas {
        1.2
    } else {
        1.0
    }
}
