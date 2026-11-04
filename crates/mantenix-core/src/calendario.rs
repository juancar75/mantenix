//! Calendario de mantenimiento de todos los contratos: visitas del plan (previstas o hechas),
//! inspecciones reglamentarias y certificados anuales. Se puede exportar a iCalendar (`.ics`)
//! para importarlo en Outlook, Teams (calendario de Outlook) o Google Calendar.

use serde::Serialize;

use crate::modelo::Contrato;
use crate::plan::Plan;
use crate::registro::{estado, fecha_civil, Registro};

/// Evento del calendario.
#[derive(Debug, Clone, Serialize)]
pub struct Evento {
    /// Fecha (AAAA-MM-DD).
    pub fecha: String,
    /// Contrato.
    pub contrato: String,
    /// `visita`, `inspeccion` o `certificado`.
    pub tipo: String,
    /// Título corto.
    pub titulo: String,
    /// Detalle.
    pub detalle: String,
    /// `hecha`, `pendiente` o `atrasada` (visitas); vacío en el resto.
    pub estado: String,
}

/// Días desde el 1-1-1970 de una fecha civil (inverso de [`fecha_civil`]).
pub fn dias_desde_civil(a: u32, m: u32, d: u32) -> i64 {
    let (a, m, d) = (a as i64, m as i64, d as i64);
    let y = if m <= 2 { a - 1 } else { a };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Primer día laborable (lunes a viernes) a partir del día `d` del mes.
pub fn laborable(a: u32, m: u32, d: u32) -> String {
    let mut n = dias_desde_civil(a, m, d);
    // 1-1-1970 fue jueves: (n + 3) % 7 → 0 = lunes … 6 = domingo.
    while (n + 3).rem_euclid(7) >= 5 {
        n += 1;
    }
    let (a, m, d) = fecha_civil(n);
    format!("{a:04}-{m:02}-{d:02}")
}

/// Eventos de un contrato. `dia` = día del mes en que se prevén las visitas (se pasa al primer
/// laborable); `hoy` = (año, mes) para marcar las atrasadas.
pub fn eventos(k: &Contrato, plan: &Plan, registros: &[Registro], dia: u32, hoy: (u32, u32)) -> Vec<Evento> {
    let mut v = Vec::new();
    let m0 = plan.visitas.first().map(|x| x.mes).unwrap_or(1);
    let anio_de = |m: u32| if m < m0 { plan.anio + 1 } else { plan.anio };
    for (vis, e) in plan.visitas.iter().zip(estado(plan, registros, hoy)) {
        let fecha = e
            .fecha
            .clone()
            .unwrap_or_else(|| laborable(anio_de(vis.mes), vis.mes, dia.clamp(1, 28)));
        let mut detalle = format!("{} h previstas en equipos.", vis.horas.to_string().replace('.', ","));
        if !vis.medidas.is_empty() {
            detalle.push_str(" Medidas de generadores (IT 3.4).");
        }
        if !vis.subcontrata.is_empty() {
            detalle.push_str(" Revisión de PCI de la subcontrata.");
        }
        if e.defectos > 0 {
            detalle.push_str(&format!(" {} defectos anotados.", e.defectos));
        }
        v.push(Evento {
            fecha,
            contrato: k.nombre.clone(),
            tipo: "visita".into(),
            titulo: format!("Visita {}/{} · {}", vis.numero, plan.n_visitas, k.nombre),
            detalle,
            estado: e.estado,
        });
    }
    for i in &k.inspecciones {
        let Some(info) = crate::inspecciones::buscar(&i.id) else {
            continue;
        };
        let Some(prox) = i.ultima.map(|u| u + info.periodicidad_anios) else {
            continue;
        };
        v.push(Evento {
            fecha: laborable(prox, m0, 1),
            contrato: k.nombre.clone(),
            tipo: "inspeccion".into(),
            titulo: format!("Inspección {} · {}", info.reglamento, k.nombre),
            detalle: format!("{} ({}). Avisar al titular y pedir oferta.", info.texto, info.agente),
            estado: String::new(),
        });
    }
    if k.instalaciones
        .iter()
        .any(|i| i.familia == "RITE" && i.potencia_kw.unwrap_or(0.0) > 70.0)
    {
        let ultimo = plan.visitas.last().map(|x| x.mes).unwrap_or(12);
        v.push(Evento {
            fecha: laborable(anio_de(ultimo), ultimo, 20),
            contrato: k.nombre.clone(),
            tipo: "certificado".into(),
            titulo: format!("Certificado anual RITE · {}", k.nombre),
            detalle: "Emitir el certificado de mantenimiento (RITE art. 28) con las visitas y consumos del registro."
                .into(),
            estado: String::new(),
        });
    }
    v
}

fn escapar(t: &str) -> String {
    t.replace('\\', "\\\\")
        .replace(';', "\\;")
        .replace(',', "\\,")
        .replace('\n', "\\n")
}

/// Calendario en formato iCalendar (RFC 5545): un evento de día completo por entrada, con aviso
/// una semana antes. Se importa en Outlook (y se ve en Teams) o en Google Calendar.
pub fn ics(eventos: &[Evento], empresa: &str) -> String {
    let mut s = String::from(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Mantenix//Mantenimiento//ES\r\nCALSCALE:GREGORIAN\r\nMETHOD:PUBLISH\r\n",
    );
    s.push_str(&format!(
        "X-WR-CALNAME:{}\r\n",
        escapar(&format!("Mantenimiento {empresa}"))
    ));
    for e in eventos {
        let f = e.fecha.replace('-', "");
        let (a, m, d) = (&e.fecha[0..4], &e.fecha[5..7], &e.fecha[8..10]);
        let sig = fecha_civil(
            dias_desde_civil(
                a.parse().unwrap_or(1970),
                m.parse().unwrap_or(1),
                d.parse().unwrap_or(1),
            ) + 1,
        );
        let uid: String = format!("{}-{}-{}", e.tipo, e.contrato, e.titulo)
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() {
                    c.to_ascii_lowercase()
                } else {
                    '-'
                }
            })
            .collect();
        s.push_str("BEGIN:VEVENT\r\n");
        s.push_str(&format!("UID:{uid}@mantenix\r\n"));
        s.push_str(&format!("DTSTAMP:{f}T000000Z\r\n"));
        s.push_str(&format!("DTSTART;VALUE=DATE:{f}\r\n"));
        s.push_str(&format!("DTEND;VALUE=DATE:{:04}{:02}{:02}\r\n", sig.0, sig.1, sig.2));
        s.push_str(&format!("SUMMARY:{}\r\n", escapar(&e.titulo)));
        s.push_str(&format!("DESCRIPTION:{}\r\n", escapar(&e.detalle)));
        s.push_str(&format!(
            "CATEGORIES:{}\r\n",
            match e.tipo.as_str() {
                "inspeccion" => "Inspección",
                "certificado" => "Certificado",
                _ => "Mantenimiento",
            }
        ));
        s.push_str("BEGIN:VALARM\r\nACTION:DISPLAY\r\nDESCRIPTION:Recordatorio\r\nTRIGGER:-P7D\r\nEND:VALARM\r\n");
        s.push_str("END:VEVENT\r\n");
    }
    s.push_str("END:VCALENDAR\r\n");
    s
}

/// Eventos de todos los contratos de la base, ordenados por fecha.
pub fn de_todos(
    c: &rusqlite::Connection,
    b: &crate::datos::Biblioteca,
    dia: u32,
    hoy: (u32, u32),
) -> Result<Vec<Evento>, crate::Error> {
    let mut v = Vec::new();
    for (nombre, _, _) in crate::db::listar_contratos(c)? {
        let (_, k) = crate::db::cargar_contrato(c, &nombre)?;
        let Ok(val) = crate::motor::valorar(b, &k) else {
            continue;
        };
        let plan = crate::plan::plan(&k, &val, crate::plan::mes_inicio(&k));
        let regs = crate::db::listar_registros(c, &nombre)?;
        v.extend(eventos(&k, &plan, &regs, dia, hoy));
    }
    v.sort_by(|a, b| a.fecha.cmp(&b.fecha).then(a.contrato.cmp(&b.contrato)));
    Ok(v)
}
