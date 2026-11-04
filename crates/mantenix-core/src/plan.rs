//! Plan anual de mantenimiento y hojas de visita.
//!
//! Reparte las operaciones de la valoración entre las visitas del año según su periodicidad:
//!
//! | Periodicidad | Visitas (de N al año) |
//! |---|---|
//! | `m`, `visita` | todas |
//! | `3m` | cada N/4 (la primera al inicio de la temporada) |
//! | `2t`, `6m` | la primera y la de mitad de periodo (al menos dos meses entre ambas) |
//! | `t`, `1a` | la primera (inicio de temporada); si es cada 2-4 años (≤70 kW), se indica |
//! | `S*` | la hace el usuario: no entra en la hoja |
//! | varios años | fuera del plan anual (ofertar aparte) |
//!
//! Añade las medidas de rendimiento de generadores de la IT 3.4.1 y 3.4.2 del RITE (tablas
//! 3.2 y 3.3 de medidas) y la lectura de contadores para el seguimiento de consumos de la
//! IT 3.4.4 en instalaciones de más de 70 kW.

use serde::Serialize;

use crate::modelo::Contrato;
use crate::motor::Valoracion;

/// Operación prevista en una visita.
#[derive(Debug, Clone, Serialize)]
pub struct OperacionVisita {
    /// Identificador de la operación.
    pub operacion_id: String,
    /// Texto de la operación.
    pub texto: String,
    /// Código de periodicidad.
    pub periodicidad: String,
    /// Nota: «cada 2 años», «limpieza: presupuesto aparte»…
    pub nota: String,
}

/// Equipo con las operaciones que tocan en una visita.
#[derive(Debug, Clone, Serialize)]
pub struct EquipoVisita {
    /// Instalación.
    pub instalacion: String,
    /// Descripción del equipo.
    pub descripcion: String,
    /// Unidades.
    pub cantidad: f64,
    /// Operaciones de esta visita.
    pub operaciones: Vec<OperacionVisita>,
}

/// Medidas de rendimiento de un tipo de generador (IT 3.4.1 / 3.4.2).
#[derive(Debug, Clone, Serialize)]
pub struct Medidas {
    /// `calor` o `frio`.
    pub tipo: String,
    /// Generadores (una fila por unidad).
    pub generadores: Vec<String>,
    /// Columnas que se miden.
    pub magnitudes: Vec<String>,
}

/// Una visita del año.
#[derive(Debug, Clone, Serialize)]
pub struct Visita {
    /// Número de visita (1..N).
    pub numero: u32,
    /// Mes previsto (1-12).
    pub mes: u32,
    /// Nombre del mes.
    pub mes_nombre: String,
    /// Equipos con sus operaciones.
    pub equipos: Vec<EquipoVisita>,
    /// Medidas de rendimiento que tocan en esta visita.
    pub medidas: Vec<Medidas>,
    /// Lectura de contadores (seguimiento de consumos, IT 3.4.4).
    pub contadores: bool,
    /// Revisión de protección contra incendios por subcontrata en esta visita.
    pub subcontrata: Vec<String>,
    /// Horas previstas (equipos, sin desplazamiento).
    pub horas: f64,
}

/// Fila del plan anual (operación × meses).
#[derive(Debug, Clone, Serialize)]
pub struct FilaPlan {
    /// Equipo.
    pub equipo: String,
    /// Operación.
    pub texto: String,
    /// Periodicidad.
    pub periodicidad: String,
    /// Números de visita en que se hace.
    pub visitas: Vec<u32>,
}

/// Plan anual con sus visitas.
#[derive(Debug, Clone, Serialize)]
pub struct Plan {
    /// Nombre del contrato.
    pub contrato: String,
    /// Año.
    pub anio: u32,
    /// Visitas al año.
    pub n_visitas: u32,
    /// Visitas.
    pub visitas: Vec<Visita>,
    /// Plan anual (para la tabla resumen).
    pub filas: Vec<FilaPlan>,
}

/// Nombres de los meses.
pub const MESES: [&str; 12] = [
    "enero",
    "febrero",
    "marzo",
    "abril",
    "mayo",
    "junio",
    "julio",
    "agosto",
    "septiembre",
    "octubre",
    "noviembre",
    "diciembre",
];

/// Visitas (índices 0..n) en que toca una periodicidad. `None` si no entra en el plan anual.
fn indices(periodicidad: &str, veces: f64, n: u32) -> Option<Vec<u32>> {
    let n = n.max(1);
    let todas = || (0..n).collect::<Vec<_>>();
    let cada = |k: u32| (0..n).step_by(k.max(1) as usize).collect::<Vec<_>>();
    match periodicidad.trim() {
        "S*" => None,
        "m" | "visita" | "S" => Some(todas()),
        "3m" => Some(if n >= 4 { cada(n / 4) } else { todas() }),
        "2t" | "2t+S*" | "6m" => Some(if n >= 2 { vec![0, n / 2] } else { vec![0] }),
        "t" | "1a" => Some(vec![0]),
        // Hasta 70 kW, todas las operaciones van en la revisión de la tabla 3.1.
        _ if veces > 0.0 => Some(vec![0]),
        _ => None,
    }
}

/// Mes (nombre en español) de un texto de fecha como «Madrid, 3 de noviembre de 2026».
pub fn mes_de_texto(t: &str) -> Option<u32> {
    let t = t.to_lowercase();
    MESES
        .iter()
        .position(|m| t.contains(&format!(" {m} ")))
        .map(|i| i as u32 + 1)
}

/// Construye el plan anual. `mes_inicio` es el mes de la primera visita (1-12).
pub fn plan(k: &Contrato, v: &Valoracion, mes_inicio: u32) -> Plan {
    let n = v.visitas.max(1);
    let mes = |i: u32| ((mes_inicio.clamp(1, 12) - 1 + i * 12 / n) % 12) + 1;
    let mut visitas: Vec<Visita> = (0..n)
        .map(|i| Visita {
            numero: i + 1,
            mes: mes(i),
            mes_nombre: MESES[(mes(i) - 1) as usize].into(),
            equipos: Vec::new(),
            medidas: Vec::new(),
            contadores: false,
            subcontrata: Vec::new(),
            horas: 0.0,
        })
        .collect();
    let mut filas = Vec::new();

    let mut poner = |instalacion: &str,
                     descripcion: &str,
                     cantidad: f64,
                     op: OperacionVisita,
                     veces: f64,
                     horas_anio: f64,
                     visitas: &mut Vec<Visita>| {
        let Some(idx) = indices(&op.periodicidad, veces, n) else {
            return;
        };
        let horas_vez = if idx.is_empty() {
            0.0
        } else {
            horas_anio / idx.len() as f64
        };
        for &i in &idx {
            let vis = &mut visitas[i as usize];
            vis.horas += horas_vez;
            match vis
                .equipos
                .iter_mut()
                .find(|e| e.instalacion == instalacion && e.descripcion == descripcion)
            {
                Some(e) => e.operaciones.push(op.clone()),
                None => vis.equipos.push(EquipoVisita {
                    instalacion: instalacion.into(),
                    descripcion: descripcion.into(),
                    cantidad,
                    operaciones: vec![op.clone()],
                }),
            }
        }
        filas.push(FilaPlan {
            equipo: format!("{descripcion} (× {})", cantidad),
            texto: op.texto.clone(),
            periodicidad: op.periodicidad.clone(),
            visitas: idx.iter().map(|i| i + 1).collect(),
        });
    };

    for l in &v.lineas {
        if l.modo == "subcontrata" {
            continue;
        }
        for o in &l.operaciones {
            let nota = if o.veces_anio > 0.0 && o.veces_anio < 1.0 {
                format!("cada {} años (tabla 3.1 del RITE)", (1.0 / o.veces_anio).round())
            } else {
                String::new()
            };
            poner(
                &l.instalacion,
                &l.descripcion,
                l.cantidad,
                OperacionVisita {
                    operacion_id: o.operacion_id.clone(),
                    texto: o.texto.clone(),
                    periodicidad: o.periodicidad.clone(),
                    nota,
                },
                o.veces_anio,
                o.horas_anio,
                &mut visitas,
            );
        }
    }
    for lp in &v.limpiezas {
        poner(
            &lp.instalacion,
            &lp.descripcion,
            lp.cantidad,
            OperacionVisita {
                operacion_id: lp.operacion_id.clone(),
                texto: lp.texto.clone(),
                periodicidad: lp.periodicidad.clone(),
                nota: "comprobar; si hay que limpiar, presupuesto aparte".into(),
            },
            lp.veces_anio,
            0.0,
            &mut visitas,
        );
    }

    // PCI por subcontrata: revisión trimestral de la subcontrata.
    let sub: Vec<String> = v
        .lineas
        .iter()
        .filter(|l| l.modo == "subcontrata")
        .map(|l| format!("{} (× {})", l.descripcion, l.cantidad))
        .collect();
    if !sub.is_empty() {
        for i in indices("3m", 1.0, n).unwrap_or_default() {
            visitas[i as usize].subcontrata = sub.clone();
        }
    }

    // Medidas de generadores (IT 3.4.1 y 3.4.2) y contadores (IT 3.4.4).
    for ins in k.instalaciones.iter().filter(|i| i.familia == "RITE") {
        let p = ins.potencia_kw.unwrap_or(0.0);
        let unidades = |cods: &[&str]| -> Vec<String> {
            ins.equipos
                .iter()
                .filter(|l| cods.contains(&l.codigo.as_str()))
                .flat_map(|l| {
                    let d = if l.descripcion.is_empty() {
                        l.codigo.clone()
                    } else {
                        l.descripcion.clone()
                    };
                    (1..=l.cantidad.max(0.0) as u32).map(move |i| format!("{d} nº {i}"))
                })
                .collect()
        };
        let calor = unidades(&["CALD-GAS", "CALD-GASOLEO", "CALD-BIO", "CALD-MURAL"]);
        let liquido_o_solido = ins
            .equipos
            .iter()
            .any(|l| l.codigo == "CALD-GASOLEO" || l.codigo == "CALD-BIO");
        if !calor.is_empty() && p > 20.0 {
            let per = if p > 1000.0 {
                "m"
            } else if p > 70.0 {
                "3m"
            } else {
                "2a"
            };
            let mut mag = vec![
                "T impulsión / retorno".to_string(),
                "T sala".into(),
                "T humos".into(),
                "CO / CO₂".into(),
                "Tiro".into(),
            ];
            if liquido_o_solido {
                mag.insert(4, "Opacidad".into());
            }
            for i in indices(per, 1.0, n).unwrap_or_default() {
                visitas[i as usize].medidas.push(Medidas {
                    tipo: if per == "2a" {
                        "calor (cada 2 años)".into()
                    } else {
                        "calor".into()
                    },
                    generadores: calor.clone(),
                    magnitudes: mag.clone(),
                });
            }
        }
        let frio = unidades(&["ENFRIADORA", "BC-GEO"]);
        if !frio.is_empty() && p > 70.0 {
            let per = if p > 1000.0 { "m" } else { "3m" };
            for i in indices(per, 1.0, n).unwrap_or_default() {
                visitas[i as usize].medidas.push(Medidas {
                    tipo: "frio".into(),
                    generadores: frio.clone(),
                    magnitudes: vec![
                        "T evaporador ent./sal.".into(),
                        "T condensador ent./sal.".into(),
                        "Δp evap. / cond.".into(),
                        "T y p evaporación".into(),
                        "T y p condensación".into(),
                        "Potencia eléctrica".into(),
                        "% carga / EER".into(),
                        "Caudales".into(),
                    ],
                });
            }
        }
        if p > 70.0 {
            for vis in visitas.iter_mut() {
                vis.contadores = true;
            }
        }
    }
    for vis in visitas.iter_mut() {
        vis.horas = (vis.horas * 100.0).round() / 100.0;
    }
    Plan {
        contrato: k.nombre.clone(),
        anio: k.anio,
        n_visitas: n,
        visitas,
        filas,
    }
}
