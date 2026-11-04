//! Registro de las operaciones de mantenimiento (RITE art. 27; RIPCI art. 21.2 y anexo II.6).
//!
//! Cada visita, aviso o reparación queda guardada con su fecha, técnico, resultado de cada
//! operación, medidas de generadores, lectura de contadores, defectos y material. El registro se
//! conserva al menos cinco años y no se borra al borrar el contrato.
//!
//! Con el registro se sabe qué visitas del plan están hechas, pendientes o atrasadas, y se
//! calculan los consumos anuales del certificado de mantenimiento (art. 28.2.d).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::plan::Plan;

/// Resultado de una operación en una visita.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Resultado {
    /// Identificador de la operación.
    pub operacion_id: String,
    /// Equipo (descripción del inventario).
    #[serde(default)]
    pub equipo: String,
    /// Texto de la operación (copia, por si cambia la normativa).
    #[serde(default)]
    pub texto: String,
    /// `B` bien, `D` defecto, `NA` no aplica; vacío = sin anotar.
    #[serde(default)]
    pub resultado: String,
    /// Observación.
    #[serde(default)]
    pub nota: String,
}

/// Medidas de un generador (IT 3.4.1 / 3.4.2): magnitud → valor.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MedidaGenerador {
    /// Generador.
    pub generador: String,
    /// Valores por magnitud.
    #[serde(default)]
    pub valores: BTreeMap<String, String>,
}

/// Una entrada del registro: visita preventiva, aviso de avería o reparación.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Registro {
    /// Identificador (se asigna al guardar si está vacío).
    #[serde(default)]
    pub id: String,
    /// Nombre del contrato.
    pub contrato: String,
    /// Año del plan.
    pub anio: u32,
    /// Número de visita del plan (0 si es un aviso o una reparación).
    #[serde(default)]
    pub numero: u32,
    /// `preventiva`, `aviso` o `reparacion`.
    #[serde(default = "preventiva")]
    pub tipo: String,
    /// Fecha (AAAA-MM-DD).
    pub fecha: String,
    /// Técnico o técnicos.
    #[serde(default)]
    pub tecnico: String,
    /// Hora de entrada.
    #[serde(default)]
    pub entrada: String,
    /// Hora de salida.
    #[serde(default)]
    pub salida: String,
    /// Resultado de cada operación.
    #[serde(default)]
    pub resultados: Vec<Resultado>,
    /// Medidas de generadores.
    #[serde(default)]
    pub medidas: Vec<MedidaGenerador>,
    /// Lectura de contadores: combustible, electricidad, agua_llenado, acs, energia_termica.
    #[serde(default)]
    pub contadores: BTreeMap<String, f64>,
    /// Defectos encontrados y acciones propuestas.
    #[serde(default)]
    pub defectos: String,
    /// Material sustituido o empleado.
    #[serde(default)]
    pub material: String,
    /// Observaciones.
    #[serde(default)]
    pub observaciones: String,
}

fn preventiva() -> String {
    "preventiva".into()
}

/// Estado de una visita del plan.
#[derive(Debug, Clone, Serialize)]
pub struct EstadoVisita {
    /// Número de visita.
    pub numero: u32,
    /// Mes previsto (1-12).
    pub mes: u32,
    /// Nombre del mes.
    pub mes_nombre: String,
    /// `hecha`, `pendiente` o `atrasada`.
    pub estado: String,
    /// Fecha en que se hizo.
    pub fecha: Option<String>,
    /// Identificador del registro.
    pub registro: Option<String>,
    /// Operaciones con defecto anotado.
    pub defectos: u32,
}

/// Fecha (año, mes) prevista de cada visita: el plan empieza en `mes_inicio` del año del
/// contrato y las visitas de meses anteriores caen al año siguiente.
fn prevista(plan: &Plan, i: usize) -> (u32, u32) {
    let m0 = plan.visitas.first().map(|v| v.mes).unwrap_or(1);
    let m = plan.visitas[i].mes;
    (if m < m0 { plan.anio + 1 } else { plan.anio }, m)
}

/// Estado de las visitas del plan según el registro. `hoy` = (año, mes).
pub fn estado(plan: &Plan, registros: &[Registro], hoy: (u32, u32)) -> Vec<EstadoVisita> {
    plan.visitas
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let r = registros
                .iter()
                .find(|r| r.tipo == "preventiva" && r.anio == plan.anio && r.numero == v.numero);
            let estado = match r {
                Some(_) => "hecha",
                None if prevista(plan, i) < hoy => "atrasada",
                None => "pendiente",
            };
            EstadoVisita {
                numero: v.numero,
                mes: v.mes,
                mes_nombre: v.mes_nombre.clone(),
                estado: estado.into(),
                fecha: r.map(|r| r.fecha.clone()),
                registro: r.map(|r| r.id.clone()),
                defectos: r
                    .map(|r| r.resultados.iter().filter(|x| x.resultado == "D").count() as u32)
                    .unwrap_or(0),
            }
        })
        .collect()
}

/// Consumos de un año a partir de las lecturas de contadores (son acumuladas): última lectura del
/// año menos la primera. Solo los contadores con al menos dos lecturas.
pub fn consumos(registros: &[Registro], anio: u32) -> BTreeMap<String, f64> {
    let mut lecturas: BTreeMap<String, Vec<(String, f64)>> = BTreeMap::new();
    for r in registros.iter().filter(|r| r.fecha.starts_with(&anio.to_string())) {
        for (k, v) in &r.contadores {
            lecturas.entry(k.clone()).or_default().push((r.fecha.clone(), *v));
        }
    }
    lecturas
        .into_iter()
        .filter(|(_, l)| l.len() >= 2)
        .map(|(k, mut l)| {
            l.sort_by(|a, b| a.0.cmp(&b.0));
            let d = l.last().unwrap().1 - l.first().unwrap().1;
            (k, (d * 100.0).round() / 100.0)
        })
        .collect()
}

/// Datos del registro para el certificado anual (art. 28.2.d): visitas preventivas hechas en el
/// año y consumos según los contadores. Vacío si no hay registro de ese año.
pub fn extra_certificado(registros: &[Registro], anio: u32) -> serde_json::Value {
    let del_anio: Vec<&Registro> = registros
        .iter()
        .filter(|r| r.fecha.starts_with(&anio.to_string()))
        .collect();
    if del_anio.is_empty() {
        return serde_json::json!({});
    }
    let visitas = del_anio.iter().filter(|r| r.tipo == "preventiva").count();
    let fmt = |x: f64| {
        let s = format!("{x:.2}").replace('.', ",");
        s.trim_end_matches(",00").to_string()
    };
    let consumos: serde_json::Map<String, serde_json::Value> = consumos(registros, anio)
        .into_iter()
        .map(|(k, v)| (k, serde_json::Value::String(fmt(v))))
        .collect();
    serde_json::json!({ "visitas": visitas, "consumos": consumos })
}

/// Fecha civil (año, mes, día) de un número de días desde el 1-1-1970 (algoritmo de H. Hinnant).
pub fn fecha_civil(dias: i64) -> (u32, u32, u32) {
    let z = dias + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    ((if m <= 2 { y + 1 } else { y }) as u32, m as u32, d as u32)
}
