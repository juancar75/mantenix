//! Motor de valoración: inventario → horas → coste → precio.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::datos::Biblioteca;
use crate::modelo::{Contrato, Instalacion};
use crate::Error;

/// Operación con medios propios valorada.
#[derive(Debug, Clone, Serialize)]
pub struct OperacionValorada {
    /// Identificador de la operación normativa.
    pub operacion_id: String,
    /// Texto de la operación.
    pub texto: String,
    /// Código de periodicidad.
    pub periodicidad: String,
    /// Intervenciones al año.
    pub veces_anio: f64,
    /// Minutos por unidad e intervención.
    pub minutos_ud: f64,
    /// Técnicos necesarios (1 = oficial; 2 = oficial + ayudante).
    pub tecnicos: u32,
    /// Horas-hombre al año.
    pub horas_anio: f64,
}

/// Resultado por línea del inventario.
#[derive(Debug, Clone, Serialize)]
pub struct LineaValorada {
    /// Título de la instalación a la que pertenece.
    pub instalacion: String,
    /// Código del tipo de equipo en el catálogo.
    pub codigo: String,
    /// Descripción (marca y modelo si se conoce).
    pub descripcion: String,
    /// Número de unidades.
    pub cantidad: f64,
    /// `propio` o `subcontrata`.
    pub modo: String,
    /// Horas-hombre al año.
    pub horas_anio: f64,
    /// Coste anual de mano de obra (€).
    pub mano_obra: f64,
    /// Coste anual de consumibles (€).
    pub consumibles: f64,
    /// Importe de subcontrata por visita (solo modo subcontrata).
    pub subcontrata_visita: f64,
    /// Operaciones con medios propios valoradas.
    pub operaciones: Vec<OperacionValorada>,
}

/// Operación de varios años que se oferta aparte.
#[derive(Debug, Clone, Serialize)]
pub struct OperacionAparte {
    /// Sistema RIPCI.
    pub sistema: String,
    /// Identificador de la operación normativa.
    pub operacion_id: String,
    /// Código de periodicidad.
    pub periodicidad: String,
    /// Texto de la operación.
    pub texto: String,
}

/// Régimen normativo aplicado a una instalación.
#[derive(Debug, Clone, Serialize)]
pub struct Regimen {
    /// Título de la instalación a la que pertenece.
    pub instalacion: String,
    /// Descripción (marca y modelo si se conoce).
    pub descripcion: String,
    /// Visitas mínimas que exige la normativa.
    pub visitas_minimas: u32,
}

/// Totales económicos de una valoración.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Totales {
    /// Horas-hombre al año en los equipos.
    pub horas_equipos: f64,
    /// Horas al año de desplazamiento y tiempo fijo por visita.
    pub horas_desplazamiento: f64,
    /// Coste anual de mano de obra (€).
    pub mano_obra: f64,
    /// Coste anual de consumibles (€).
    pub consumibles: f64,
    /// Coste anual de desplazamiento y tiempo fijo por visita (€).
    pub desplazamiento: f64,
    /// Coste anual de subcontrata (€).
    pub subcontrata: f64,
    /// Coste anual de los avisos incluidos en la cuota (€).
    #[serde(default)]
    pub avisos: f64,
    /// Coste directo anual (€).
    pub directo: f64,
    /// Gastos indirectos sobre el coste directo (€).
    pub indirectos: f64,
    /// Contingencia sobre el coste directo (€).
    pub contingencia: f64,
    /// Coste total anual: directo + indirectos + contingencia (€).
    pub coste_total: f64,
    /// Margen sobre el precio de venta (%).
    pub margen_pct: f64,
    /// Precio de venta anual (€).
    pub precio_anual: f64,
    /// Precio de venta por visita (€).
    pub precio_visita: f64,
    /// Precio al que está vendido el contrato (€/año), para comparar.
    pub precio_actual_anual: Option<f64>,
    /// Margen sobre venta que deja el precio actual con los costes calculados.
    pub margen_precio_actual_pct: Option<f64>,
}

/// Resultado completo de valorar un contrato.
#[derive(Debug, Clone, Serialize)]
pub struct Valoracion {
    /// Nombre único del contrato.
    pub nombre: String,
    /// Año de los parámetros y tarifas aplicados.
    pub anio: u32,
    /// `viviendas` u `otros` (columna de la tabla 3.1).
    pub uso: String,
    /// Visitas al año valoradas.
    pub visitas: u32,
    /// Visitas mínimas que exige la normativa.
    pub visitas_minimas: u32,
    /// Visitas al año de la subcontrata.
    pub visitas_subcontrata: u32,
    /// Régimen normativo de cada instalación.
    pub regimenes: Vec<Regimen>,
    /// Valoración de cada línea del inventario.
    pub lineas: Vec<LineaValorada>,
    /// Suma de partidas de subcontrata por visita (€).
    pub subcontrata_visita_suma: f64,
    /// Subcontrata facturada por visita, con el mínimo aplicado (€).
    pub subcontrata_visita_facturada: f64,
    /// Operaciones de periodicidad superior a un año (fuera de la cuota).
    pub operaciones_aparte: Vec<OperacionAparte>,
    /// Parámetros económicos usados.
    pub parametros: BTreeMap<String, f64>,
    /// Totales económicos.
    pub totales: Totales,
    /// Atención de avisos de avería.
    pub avisos_averia: AvisosValorados,
    /// Advertencias para revisar antes de ofertar (datos que faltan, visitas insuficientes…).
    pub avisos: Vec<String>,
}

/// Estimación y coste de los avisos de avería.
#[derive(Debug, Clone, Default, Serialize)]
pub struct AvisosValorados {
    /// `bolsa`, `incluidos` o `aparte`.
    pub modalidad: String,
    /// Avisos estimados al año.
    pub estimados: f64,
    /// Avisos incluidos en la cuota (en `incluidos`, los estimados redondeados).
    pub incluidos: u32,
    /// Coste de un aviso: horas en el edificio, desplazamiento y km (€).
    pub coste_aviso: f64,
    /// Coste anual incluido en la cuota (€).
    pub coste_anual: f64,
}

/// Veces al año de una operación según su código de periodicidad.
pub fn veces_anio(periodicidad: &str, temporada_meses: f64, visitas: u32) -> f64 {
    match periodicidad.trim() {
        "m" => temporada_meses,
        "t" | "1a" => 1.0,
        "2t" | "2t+S*" | "6m" => 2.0,
        "3m" => 4.0,
        "S" => 52.0,
        "S*" => 0.0, // la hace el usuario
        "visita" => visitas as f64,
        _ => 0.0, // periodicidades de varios años: fuera de cuota
    }
}

fn r2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

/// Visitas mínimas que exige la normativa para una instalación.
fn regimen(b: &Biblioteca, ins: &Instalacion, viviendas: bool) -> (String, u32) {
    match ins.familia.as_str() {
        "RITE" => {
            let p = ins.potencia_kw.unwrap_or(0.0);
            if p <= 70.0 {
                let mut revis_anio: f64 = 0.0;
                for l in &ins.equipos {
                    if let Some(eq) = b.catalogo.get(&l.codigo) {
                        if let Some(m) = b.meses_tabla_3_1(&eq.equipo_tabla_3_1, p, viviendas) {
                            revis_anio = revis_anio.max(12.0 / m);
                        }
                    }
                }
                let min = revis_anio.ceil().max(1.0) as u32;
                (format!("RITE ≤70 kW ({p} kW): revisión según tabla 3.1; operaciones de temporada con esa frecuencia, el resto en cada visita contratada"), min)
            } else {
                let mut max_v: f64 = 1.0;
                for l in &ins.equipos {
                    for t in b.tiempos_de(&l.codigo) {
                        if let Some(op) = b.operaciones.get(&t.operacion_id) {
                            let v = veces_anio(&op.periodicidad, ins.temporada_meses, 1);
                            if v < 52.0 {
                                max_v = max_v.max(v);
                            }
                        }
                    }
                }
                (format!("RITE >70 kW ({p} kW): periodicidades de la tabla 3.3; contrato obligatorio y certificado anual"), max_v.ceil() as u32)
            }
        }
        "PCI" => (
            "RIPCI anexo II: revisiones trimestrales (tabla I) y anual (tablas I y II)".into(),
            4,
        ),
        otra => (
            format!("{otra}: mantenimiento según fabricante, en cada visita contratada"),
            1,
        ),
    }
}

/// Valora un contrato: régimen normativo, horas, costes y precio de venta.
///
/// Ver el método completo en `docs/metodo-calculo.md`.
pub fn valorar(b: &Biblioteca, c: &Contrato) -> Result<Valoracion, Error> {
    let anio = c.anio;
    let p = |n: &str| b.parametro(anio, n);
    let coste_of = p("coste_hora_oficial")?;
    let coste_ay = p("coste_hora_ayudante")?;
    let coste_km = p("coste_km")?;
    let fijo_min = p("tiempo_fijo_visita_min")?;
    let ind_pct = p("indirectos_pct")?;
    let cont_pct = p("contingencia_pct")?;
    let margen_pct = match c.margen_pct {
        Some(m) => m,
        None => p("margen_pct")?,
    };
    let minimo_sub = p("minimo_visita_subcontrata")?;
    let viviendas = c.uso == "viviendas";

    let mut avisos = Vec::new();
    let mut regimenes = Vec::new();
    let mut visitas_minimas = 1;
    for ins in &c.instalaciones {
        let (d, v) = regimen(b, ins, viviendas);
        visitas_minimas = visitas_minimas.max(v);
        regimenes.push(Regimen {
            instalacion: ins.titulo.clone(),
            descripcion: d,
            visitas_minimas: v,
        });
    }
    let visitas = c.visitas_contratadas.unwrap_or(visitas_minimas);
    if visitas < visitas_minimas {
        avisos.push(format!(
            "Visitas contratadas ({visitas}) por debajo del mínimo normativo ({visitas_minimas})."
        ));
    }

    let mut lineas = Vec::new();
    let mut aparte: BTreeMap<String, OperacionAparte> = BTreeMap::new();
    let mut hay_propio = false;
    let mut visitas_sub = 0u32;

    for ins in &c.instalaciones {
        let rite_le70 = ins.familia == "RITE" && ins.potencia_kw.unwrap_or(0.0) <= 70.0;
        for l in &ins.equipos {
            let Some(eq) = b.catalogo.get(&l.codigo) else {
                avisos.push(format!("Equipo «{}» no está en el catálogo: no se valora.", l.codigo));
                continue;
            };
            let modo = l.modo.clone().unwrap_or_else(|| eq.modo_defecto.clone());
            let descripcion = if l.descripcion.is_empty() {
                eq.descripcion.clone()
            } else {
                l.descripcion.clone()
            };
            let mut linea = LineaValorada {
                instalacion: ins.titulo.clone(),
                codigo: l.codigo.clone(),
                descripcion,
                cantidad: l.cantidad,
                modo: modo.clone(),
                horas_anio: 0.0,
                mano_obra: 0.0,
                consumibles: 0.0,
                subcontrata_visita: 0.0,
                operaciones: Vec::new(),
            };

            // Operaciones de varios años (RIPCI): se listan aparte, sea cual sea el modo.
            if !eq.sistema_ripci.is_empty() {
                for op in b
                    .operaciones
                    .values()
                    .filter(|o| o.sistema == eq.sistema_ripci && o.tipo != "preventivo")
                {
                    aparte.entry(op.id.clone()).or_insert(OperacionAparte {
                        sistema: eq.sistema_ripci.clone(),
                        operacion_id: op.id.clone(),
                        periodicidad: op.periodicidad.clone(),
                        texto: op.texto.clone(),
                    });
                }
            }

            if modo == "subcontrata" {
                visitas_sub = visitas_sub.max(4);
                match b.partida(&eq.partida_subcontrata, anio) {
                    Some(pt) if pt.tipo == "incluido" => {}
                    Some(pt) if pt.tipo == "preventivo" => match pt.precio {
                        Some(pr) => linea.subcontrata_visita = r2(pr * l.cantidad),
                        None => avisos.push(format!(
                            "{}: partida {} sin precio en la tarifa de {}.",
                            l.codigo, pt.partida, pt.proveedor
                        )),
                    },
                    Some(pt) => avisos.push(format!(
                        "{}: partida {} es «{}», no tiene precio por revisión.",
                        l.codigo, pt.partida, pt.tipo
                    )),
                    None => avisos.push(format!(
                        "{}: sin partida de subcontrata; valorar o pasar a medios propios.",
                        l.codigo
                    )),
                }
            } else {
                hay_propio = true;
                let potencia = ins.potencia_kw.unwrap_or(0.0);
                let rev_t31 = b
                    .meses_tabla_3_1(&eq.equipo_tabla_3_1, potencia, viviendas)
                    .map(|m| (12.0 / m).min(1.0))
                    .unwrap_or(1.0);
                let tiempos: Vec<_> = b.tiempos_de(&l.codigo).collect();
                if tiempos.is_empty() {
                    avisos.push(format!(
                        "{}: sin tiempos estándar; no se valora la mano de obra.",
                        l.codigo
                    ));
                }
                for t in tiempos {
                    let Some(op) = b.operaciones.get(&t.operacion_id) else {
                        avisos.push(format!(
                            "{}: operación {} no existe en la biblioteca.",
                            l.codigo, t.operacion_id
                        ));
                        continue;
                    };
                    if op.tipo != "preventivo" {
                        continue;
                    }
                    let base = veces_anio(&op.periodicidad, ins.temporada_meses, visitas);
                    let mut veces = if rite_le70 && op.periodicidad == "t" {
                        // ≤70 kW: las operaciones de temporada siguen la frecuencia de la tabla 3.1
                        // (una vez al año, o cada 2-4 años según equipo y uso).
                        rev_t31
                    } else {
                        base
                    };
                    if veces > visitas as f64 && op.periodicidad != "S" {
                        // No se pueden hacer más intervenciones que visitas.
                        veces = visitas as f64;
                    }
                    let horas = t.minutos / 60.0 * l.cantidad * veces;
                    let coste = horas * (coste_of + (t.tecnicos.saturating_sub(1)) as f64 * coste_ay);
                    linea.horas_anio += horas * t.tecnicos as f64;
                    linea.mano_obra += coste;
                    linea.consumibles += t.consumibles_eur * l.cantidad * veces;
                    linea.operaciones.push(OperacionValorada {
                        operacion_id: op.id.clone(),
                        texto: op.texto.clone(),
                        periodicidad: op.periodicidad.clone(),
                        veces_anio: veces,
                        minutos_ud: t.minutos,
                        tecnicos: t.tecnicos,
                        horas_anio: r2(horas * t.tecnicos as f64),
                    });
                }
                linea.horas_anio = r2(linea.horas_anio);
                linea.mano_obra = r2(linea.mano_obra);
                linea.consumibles = r2(linea.consumibles);
            }
            lineas.push(linea);
        }
    }

    // Avisos de avería: estimación por equipo y coste de los incluidos en la cuota.
    let mut estimados = 0.0;
    for ins in &c.instalaciones {
        for l in &ins.equipos {
            let familia = b.catalogo.get(&l.codigo).map(|e| e.familia.as_str()).unwrap_or("");
            estimados += l.cantidad * crate::avisos::tasa(&l.codigo, familia);
        }
    }
    let av = &c.avisos;
    estimados *=
        crate::avisos::factor_antiguedad(&av.antiguedad) * crate::avisos::factor_uso(viviendas, av.servicio_24h);
    let estimados = r2(estimados);
    let coste_aviso = av.horas_por_aviso * coste_of
        + 2.0 * c.desplazamiento.minutos_ida / 60.0 * coste_of
        + 2.0 * c.desplazamiento.km_ida * coste_km;
    let incluidos = match av.modalidad.as_str() {
        "aparte" => 0,
        "incluidos" => estimados.ceil() as u32,
        _ => av.bolsa.unwrap_or(estimados.ceil() as u32),
    };
    let coste_avisos = incluidos as f64 * coste_aviso;
    if av.modalidad == "bolsa" && av.bolsa.is_some_and(|n| (n as f64) < estimados) {
        avisos.push(format!(
            "La bolsa de {} avisos es menor que los {estimados} estimados: el exceso se facturará aparte.",
            av.bolsa.unwrap_or(0)
        ));
    }

    // Subcontrata: suma por visita con mínimo de facturación.
    let sub_suma: f64 = lineas.iter().map(|l| l.subcontrata_visita).sum();
    let sub_fact = if sub_suma > 0.0 { sub_suma.max(minimo_sub) } else { 0.0 };
    let subcontrata = sub_fact * visitas_sub as f64;

    // Desplazamiento y tiempo fijo por visita (solo si hay trabajo propio).
    let (horas_desp, coste_desp) = if hay_propio {
        let h = visitas as f64 * (2.0 * c.desplazamiento.minutos_ida + fijo_min) / 60.0;
        let km = visitas as f64 * 2.0 * c.desplazamiento.km_ida;
        (h, h * coste_of + km * coste_km)
    } else {
        (0.0, 0.0)
    };
    if hay_propio && c.desplazamiento.km_ida == 0.0 {
        avisos
            .push("Desplazamiento sin indicar (km y minutos de ida): solo se cuenta el tiempo fijo por visita.".into());
    }

    let mano_obra: f64 = lineas.iter().map(|l| l.mano_obra).sum();
    let consumibles: f64 = lineas.iter().map(|l| l.consumibles).sum();
    let horas_equipos: f64 = lineas.iter().map(|l| l.horas_anio).sum();
    let directo = mano_obra + consumibles + coste_desp + subcontrata + coste_avisos;
    let indirectos = directo * ind_pct / 100.0;
    let contingencia = directo * cont_pct / 100.0;
    let coste_total = directo + indirectos + contingencia;
    if margen_pct >= 100.0 {
        return Err(Error::Margen(margen_pct));
    }
    let precio_anual = coste_total / (1.0 - margen_pct / 100.0);
    let margen_actual = c.precio_actual_anual.map(|pa| r2((1.0 - coste_total / pa) * 100.0));

    let mut parametros = BTreeMap::new();
    for (k, v) in [
        ("coste_hora_oficial", coste_of),
        ("coste_hora_ayudante", coste_ay),
        ("coste_km", coste_km),
        ("tiempo_fijo_visita_min", fijo_min),
        ("indirectos_pct", ind_pct),
        ("contingencia_pct", cont_pct),
        ("margen_pct", margen_pct),
        ("minimo_visita_subcontrata", minimo_sub),
    ] {
        parametros.insert(k.to_string(), v);
    }

    Ok(Valoracion {
        nombre: c.nombre.clone(),
        anio,
        uso: c.uso.clone(),
        visitas,
        visitas_minimas,
        visitas_subcontrata: visitas_sub,
        regimenes,
        lineas,
        subcontrata_visita_suma: r2(sub_suma),
        subcontrata_visita_facturada: r2(sub_fact),
        operaciones_aparte: aparte.into_values().collect(),
        parametros,
        avisos_averia: AvisosValorados {
            modalidad: av.modalidad.clone(),
            estimados,
            incluidos,
            coste_aviso: r2(coste_aviso),
            coste_anual: r2(coste_avisos),
        },
        totales: Totales {
            horas_equipos: r2(horas_equipos),
            horas_desplazamiento: r2(horas_desp),
            mano_obra: r2(mano_obra),
            consumibles: r2(consumibles),
            desplazamiento: r2(coste_desp),
            subcontrata: r2(subcontrata),
            avisos: r2(coste_avisos),
            directo: r2(directo),
            indirectos: r2(indirectos),
            contingencia: r2(contingencia),
            coste_total: r2(coste_total),
            margen_pct,
            precio_anual: r2(precio_anual),
            precio_visita: r2(precio_anual / visitas.max(1) as f64),
            precio_actual_anual: c.precio_actual_anual,
            margen_precio_actual_pct: margen_actual,
        },
        avisos,
    })
}
