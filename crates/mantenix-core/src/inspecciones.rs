//! Inspecciones y revisiones reglamentarias periódicas (OCA/EICI, distribuidora, empresa
//! habilitada): RITE, RIPCI, REBT, gas y productos petrolíferos.
//!
//! No forman parte del mantenimiento preventivo ni de la cuota: se valoran aparte. La empresa
//! mantenedora avisa de la fecha, coordina con el organismo que elija el titular, acompaña la
//! inspección y presupuesta la subsanación de defectos.
//!
//! La tabla está en `datos/semilla/normativa/inspecciones_periodicas.csv` y va incluida en el
//! programa (es normativa, igual para todas las empresas).

use serde::{Deserialize, Serialize};

use crate::datos::Biblioteca;
use crate::modelo::Contrato;

const CSV: &str = include_str!("../../../datos/semilla/normativa/inspecciones_periodicas.csv");

/// Inspección o revisión reglamentaria periódica.
#[derive(Debug, Clone, Serialize)]
pub struct InspeccionPeriodica {
    /// Identificador estable (`INSP-…`).
    pub id: String,
    /// Reglamento y artículo o instrucción.
    pub reglamento: String,
    /// Instalación a la que se refiere.
    pub instalacion: String,
    /// Cuándo es obligatoria.
    pub condicion: String,
    /// Periodicidad en años.
    pub periodicidad_anios: u32,
    /// Quién la hace.
    pub agente: String,
    /// Texto para el contrato.
    pub texto: String,
    /// Fuente del dato.
    pub fuente: String,
    /// `no` mientras no se coteje con el reglamento.
    pub verificado: String,
}

/// Inspección reglamentaria incluida en un contrato (para avisar y valorar aparte).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InspeccionContrato {
    /// Identificador de [`InspeccionPeriodica`].
    pub id: String,
    /// Año de la última inspección, si se conoce.
    #[serde(default)]
    pub ultima: Option<u32>,
    /// Precio del organismo o de la distribuidora (€, sin IVA), si se conoce. Se repercute tal cual.
    #[serde(default)]
    pub precio_organismo: Option<f64>,
}

/// Inspección valorada: precio orientativo, fuera de la cuota.
#[derive(Debug, Clone, Serialize)]
pub struct InspeccionValorada {
    /// Identificador.
    pub id: String,
    /// Reglamento.
    pub reglamento: String,
    /// Texto para el contrato.
    pub texto: String,
    /// Quién la hace.
    pub agente: String,
    /// Periodicidad en años.
    pub periodicidad_anios: u32,
    /// Año de la última, si se conoce.
    pub ultima: Option<u32>,
    /// Año de la próxima (última + periodicidad), si se conoce la última.
    pub proxima: Option<u32>,
    /// Horas de la empresa mantenedora: preparación, acompañamiento y gestión.
    pub horas_acompanamiento: f64,
    /// Precio de esas horas (€).
    pub precio_acompanamiento: f64,
    /// Precio del organismo o distribuidora (€), si se conoce.
    pub precio_organismo: Option<f64>,
    /// Total orientativo por inspección (€).
    pub precio_total: f64,
}

/// Tabla de inspecciones periódicas.
pub fn catalogo() -> Vec<InspeccionPeriodica> {
    CSV.trim_start_matches('\u{feff}')
        .lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| {
            let c: Vec<&str> = l.trim_end_matches('\r').split(';').collect();
            if c.len() < 9 {
                return None;
            }
            Some(InspeccionPeriodica {
                id: c[0].into(),
                reglamento: c[1].into(),
                instalacion: c[2].into(),
                condicion: c[3].into(),
                periodicidad_anios: c[4].trim().parse().ok()?,
                agente: c[5].into(),
                texto: c[6].into(),
                fuente: c[7].into(),
                verificado: c[8].into(),
            })
        })
        .collect()
}

/// Inspección por identificador.
pub fn buscar(id: &str) -> Option<InspeccionPeriodica> {
    catalogo().into_iter().find(|i| i.id == id)
}

/// Inspecciones que probablemente corresponden al edificio según su inventario. Son una
/// sugerencia: la obligación depende de datos que el inventario no tiene (superficie del garaje,
/// potencia eléctrica, capacidad del depósito…).
pub fn sugeridas(b: &Biblioteca, k: &Contrato) -> Vec<String> {
    let hay = |codigos: &[&str]| {
        k.instalaciones
            .iter()
            .flat_map(|i| &i.equipos)
            .any(|l| l.cantidad > 0.0 && codigos.contains(&l.codigo.as_str()))
    };
    let garaje = hay(&[
        "VENT-GAR",
        "CO-CENTRAL",
        "CO-DET",
        "SCTEH-SIS",
        "SCTEH-VENT",
        "SCH-BOMB",
    ]);
    let mut v = Vec::new();
    if k.instalaciones
        .iter()
        .any(|i| i.familia == "RITE" && i.potencia_kw.unwrap_or(0.0) > 70.0)
    {
        v.extend(["INSP-RITE-PARC", "INSP-RITE-COMP"]);
    }
    let hay_pci = k.instalaciones.iter().flat_map(|i| &i.equipos).any(|l| {
        b.catalogo.get(&l.codigo).is_some_and(|e| {
            e.familia == "PCI"
                && !e.sistema_ripci.is_empty()
                && e.sistema_ripci != "CO"
                && e.sistema_ripci != "VENT"
                && e.sistema_ripci != "AE"
        })
    });
    if hay_pci && (garaje || k.uso != "viviendas") {
        v.push("INSP-RIPCI");
    }
    if garaje || hay(&["CALD-GAS"]) {
        v.push("INSP-REBT-RIESGO");
    }
    if k.uso == "viviendas" {
        v.push("INSP-REBT-ZC");
    }
    if hay(&["CALD-GAS", "CALD-MURAL", "CALENT-GAS"]) {
        v.push("INSP-GAS-RED");
    }
    if hay(&["CALD-GASOLEO"]) {
        v.extend(["INSP-IP03-REV", "INSP-IP03-INSP"]);
    }
    v.into_iter().map(String::from).collect()
}

/// Valora las inspecciones del contrato: horas de acompañamiento al precio de venta de la hora
/// de oficial, más el precio del organismo si se conoce.
pub fn valorar(k: &Contrato, horas: f64, precio_hora: f64) -> Vec<InspeccionValorada> {
    k.inspecciones
        .iter()
        .filter_map(|ic| {
            let i = buscar(&ic.id)?;
            let acomp = (horas * precio_hora * 100.0).round() / 100.0;
            Some(InspeccionValorada {
                proxima: ic.ultima.map(|u| u + i.periodicidad_anios),
                precio_total: acomp + ic.precio_organismo.unwrap_or(0.0),
                id: i.id,
                reglamento: i.reglamento,
                texto: i.texto,
                agente: i.agente,
                periodicidad_anios: i.periodicidad_anios,
                ultima: ic.ultima,
                horas_acompanamiento: horas,
                precio_acompanamiento: acomp,
                precio_organismo: ic.precio_organismo,
            })
        })
        .collect()
}
