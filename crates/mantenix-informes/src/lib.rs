//! # Mantenix — informes PDF
//!
//! Convierte los resultados del núcleo en documentos PDF con [Typst](https://typst.app).
//!
//! * Las plantillas (`plantillas/*.typ`) y la tipografía Inter (`assets/fuentes/`) se **incrustan
//!   en el ejecutable** al compilar, de modo que la aplicación genera los mismos PDF en cualquier
//!   equipo, sin depender de las fuentes instaladas.
//! * Los datos se pasan a cada plantilla como texto JSON en `sys.inputs` (no se escriben ficheros
//!   intermedios).
//!
//! Para cambiar el aspecto de un documento solo hay que editar su plantilla `.typ`; este crate no
//! contiene maquetación.

#![warn(missing_docs)]

use mantenix_core::datos::Operacion;
use mantenix_core::presupuesto::calcular;
use mantenix_core::{Presupuesto, Tarifa, Valoracion};
use typst::foundations::{Dict, IntoValue};
use typst_as_lib::TypstEngine;
use typst_layout::PagedDocument;

/// Plantillas incrustadas: (nombre de fichero, contenido).
const PLANTILLAS: [(&str, &str); 5] = [
    ("comun.typ", include_str!("../../../plantillas/comun.typ")),
    ("hoja_costes.typ", include_str!("../../../plantillas/hoja_costes.typ")),
    ("tarifa.typ", include_str!("../../../plantillas/tarifa.typ")),
    ("contrato.typ", include_str!("../../../plantillas/contrato.typ")),
    ("presupuesto.typ", include_str!("../../../plantillas/presupuesto.typ")),
];

/// Tipografía Inter (SIL OFL 1.1, ver `assets/fuentes/OFL.txt`).
const FUENTES: [&[u8]; 4] = [
    include_bytes!("../../../assets/fuentes/Inter-Regular.otf"),
    include_bytes!("../../../assets/fuentes/Inter-Bold.otf"),
    include_bytes!("../../../assets/fuentes/Inter-Italic.otf"),
    include_bytes!("../../../assets/fuentes/Inter-BoldItalic.otf"),
];

/// Errores al generar un documento.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error de compilación de la plantilla Typst (normalmente un dato que falta en el JSON).
    #[error("Error en la plantilla {0}: {1}")]
    Plantilla(String, String),
    /// Error al convertir a PDF.
    #[error("Error generando el PDF: {0}")]
    Pdf(String),
    /// Error al serializar los datos.
    #[error("Error preparando los datos: {0}")]
    Datos(#[from] serde_json::Error),
}

/// Datos de la empresa que aparecen en los documentos (cabeceras, contrato).
pub type Empresa = serde_json::Value;

/// Compila una plantilla con las entradas indicadas (nombre → JSON en texto) y devuelve el PDF.
fn compilar(plantilla: &str, entradas: &[(&str, String)]) -> Result<Vec<u8>, Error> {
    let motor = TypstEngine::builder()
        .fonts(FUENTES)
        .with_static_source_file_resolver(PLANTILLAS)
        .build();
    let mut inputs = Dict::new();
    for (k, v) in entradas {
        inputs.insert((*k).into(), v.as_str().into_value());
    }
    let doc: PagedDocument = motor
        .compile_with_input(plantilla, inputs)
        .output
        .map_err(|e| Error::Plantilla(plantilla.into(), format!("{e:?}")))?;
    typst_pdf::pdf(&doc, &Default::default()).map_err(|e| Error::Pdf(format!("{e:?}")))
}

/// Hoja de costes interna de una valoración.
pub fn hoja_costes(v: &Valoracion, empresa: &Empresa) -> Result<Vec<u8>, Error> {
    compilar(
        "hoja_costes.typ",
        &[("datos", serde_json::to_string(v)?), ("empresa", empresa.to_string())],
    )
}

/// Tarifa propia de un año.
pub fn tarifa(t: &Tarifa, empresa: &Empresa) -> Result<Vec<u8>, Error> {
    compilar(
        "tarifa.typ",
        &[("datos", serde_json::to_string(t)?), ("empresa", empresa.to_string())],
    )
}

/// Carta presupuesto para el cliente (sin costes internos).
pub fn presupuesto(p: &Presupuesto, empresa: &Empresa) -> Result<Vec<u8>, Error> {
    let datos = serde_json::json!({ "p": p, "t": calcular(p) });
    compilar(
        "presupuesto.typ",
        &[("datos", datos.to_string()), ("empresa", empresa.to_string())],
    )
}

/// Operación tal como la necesita la plantilla del contrato.
#[derive(serde::Serialize)]
struct OpContrato<'a> {
    id: &'a str,
    sistema: &'a str,
    periodicidad: &'a str,
    texto: &'a str,
    tipo: &'a str,
}

/// Contrato de mantenimiento.
///
/// * `datos`: partes, edificio, sistemas e inventario, economía y condiciones.
/// * `operaciones`: operaciones de la base de datos; la plantilla toma las del RIPCI de los
///   sistemas del contrato.
pub fn contrato<'a>(
    datos: &serde_json::Value,
    empresa: &Empresa,
    operaciones: impl IntoIterator<Item = &'a Operacion>,
) -> Result<Vec<u8>, Error> {
    let mut ops: Vec<OpContrato> = operaciones
        .into_iter()
        .filter(|o| o.id.starts_with("RIPCI"))
        .map(|o| OpContrato {
            id: &o.id,
            sistema: &o.sistema,
            periodicidad: &o.periodicidad,
            texto: &o.texto,
            tipo: &o.tipo,
        })
        .collect();
    // Orden estable (tabla y número), para que el contrato siempre salga igual.
    ops.sort_by(|a, b| a.id.cmp(b.id));
    compilar(
        "contrato.typ",
        &[
            ("datos", datos.to_string()),
            ("empresa", empresa.to_string()),
            ("operaciones", serde_json::to_string(&ops)?),
        ],
    )
}
