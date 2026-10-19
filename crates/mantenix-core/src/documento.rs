//! Datos del documento de contrato a partir del inventario y de su valoración.
//!
//! El contrato se genera con `plantillas/contrato.typ`. Esta función prepara todo lo que la
//! plantilla necesita, de modo que el documento sale **del mismo inventario y del mismo precio**
//! que la valoración:
//!
//! * Instalaciones RITE: régimen, inventario y operaciones con su periodicidad (las de la
//!   valoración, es decir, las de la tabla 3.3 o las de la tabla 3.1 según la potencia).
//! * Protección contra incendios: sistemas del anexo II del RIPCI con su inventario (las
//!   operaciones las añade la plantilla desde la base de datos).
//! * Ventilación por CO y alumbrado de emergencia: inventario y operaciones del fabricante.
//! * Visitas, importe, pago según la periodicidad de las visitas y bolsa de avisos.
//!
//! Los datos de las partes (cliente, edificio, referencia, firmantes) y las condiciones que se
//! quieran cambiar se pasan en `partes` (JSON). Las condiciones que falten toman un texto por
//! defecto.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::datos::Biblioteca;
use crate::modelo::Contrato;
use crate::motor::Valoracion;

/// Importe con formato español: `1.234,56 €`.
pub fn euros(x: f64) -> String {
    let c = (x * 100.0).round() as i64;
    let (ent, dec) = (c / 100, (c % 100).abs());
    let s = ent.abs().to_string();
    let mut g = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            g.push('.');
        }
        g.push(ch);
    }
    format!("{}{g},{dec:02} €", if ent < 0 { "-" } else { "" })
}

const UNIDADES: [&str; 30] = [
    "cero",
    "uno",
    "dos",
    "tres",
    "cuatro",
    "cinco",
    "seis",
    "siete",
    "ocho",
    "nueve",
    "diez",
    "once",
    "doce",
    "trece",
    "catorce",
    "quince",
    "dieciséis",
    "diecisiete",
    "dieciocho",
    "diecinueve",
    "veinte",
    "veintiuno",
    "veintidós",
    "veintitrés",
    "veinticuatro",
    "veinticinco",
    "veintiséis",
    "veintisiete",
    "veintiocho",
    "veintinueve",
];
const DECENAS: [&str; 10] = [
    "",
    "",
    "",
    "treinta",
    "cuarenta",
    "cincuenta",
    "sesenta",
    "setenta",
    "ochenta",
    "noventa",
];
const CENTENAS: [&str; 10] = [
    "",
    "ciento",
    "doscientos",
    "trescientos",
    "cuatrocientos",
    "quinientos",
    "seiscientos",
    "setecientos",
    "ochocientos",
    "novecientos",
];

fn menor_mil(n: u64) -> String {
    if n == 100 {
        return "cien".into();
    }
    let (c, r) = (n / 100, n % 100);
    let resto = if r < 30 {
        if r == 0 {
            String::new()
        } else {
            UNIDADES[r as usize].to_string()
        }
    } else {
        let (d, u) = (r / 10, r % 10);
        if u == 0 {
            DECENAS[d as usize].to_string()
        } else {
            format!("{} y {}", DECENAS[d as usize], UNIDADES[u as usize])
        }
    };
    match (c, resto.is_empty()) {
        (0, _) => resto,
        (_, true) => CENTENAS[c as usize].to_string(),
        _ => format!("{} {resto}", CENTENAS[c as usize]),
    }
}

/// Número entero en letras («mil doscientos treinta y cuatro»). Hasta 999.999.999.
pub fn en_letras(n: u64) -> String {
    if n == 0 {
        return "cero".into();
    }
    let (millones, miles, resto) = (n / 1_000_000, (n / 1000) % 1000, n % 1000);
    let mut partes = Vec::new();
    match millones {
        0 => {}
        1 => partes.push("un millón".to_string()),
        m => partes.push(format!("{} millones", menor_mil(m).replace("uno", "un"))),
    }
    match miles {
        0 => {}
        1 => partes.push("mil".into()),
        m => partes.push(format!("{} mil", apocope(&menor_mil(m)))),
    }
    if resto > 0 {
        partes.push(menor_mil(resto));
    }
    partes.join(" ")
}

/// «veintiuno mil» → «veintiún mil», «ciento uno mil» → «ciento un mil».
fn apocope(s: &str) -> String {
    if let Some(r) = s.strip_suffix("veintiuno") {
        format!("{r}veintiún")
    } else if let Some(r) = s.strip_suffix("uno") {
        format!("{r}un")
    } else {
        s.to_string()
    }
}

/// Importe en letras: «dos mil euros con cincuenta céntimos».
pub fn euros_en_letras(x: f64) -> String {
    let c = (x * 100.0).round() as u64;
    let (e, cent) = (c / 100, c % 100);
    let base = format!("{} euros", apocope(&en_letras(e)));
    if cent == 0 {
        base
    } else {
        format!("{base} con {} céntimos", en_letras(cent))
    }
}

/// Periodicidad de facturación según las visitas al año.
pub fn periodo_pago(visitas: u32) -> &'static str {
    match visitas {
        12 => "mensual",
        6 => "bimestral",
        4 => "trimestral",
        3 => "cuatrimestral",
        2 => "semestral",
        1 => "anual",
        _ => "por visita",
    }
}

/// Texto de la periodicidad de una operación RITE en el contrato.
fn periodicidad_rite(p: &str, veces: f64) -> (u32, String) {
    match p {
        "S" => (0, "Semanal".into()),
        "m" => (1, "Mensual, durante la temporada de uso".into()),
        "3m" => (2, "Cada tres meses".into()),
        "visita" => (3, "En cada visita".into()),
        "2t" | "2t+S*" | "6m" => (4, "Dos veces por temporada".into()),
        "t" | "1a" if veces >= 1.0 => (5, "Una vez por temporada (anual)".into()),
        "t" | "1a" if veces > 0.0 => {
            let anios = (1.0 / veces).round() as u32;
            (6, format!("Cada {anios} años (tabla 3.1 del RITE)"))
        }
        _ => (9, format!("Periodicidad {p}")),
    }
}

/// Sistema del RIPCI de un equipo, agrupando la detección en un solo sistema.
fn sistema_ripci(codigo: &str) -> &str {
    if codigo.starts_with("DET") {
        "DET"
    } else {
        codigo
    }
}

fn titulo_sistema(s: &str) -> &'static str {
    match s {
        "EXT" => "Extintores de incendio",
        "DET" => "Sistema de detección y alarma de incendios",
        "BIE" => "Bocas de incendio equipadas",
        "ABAST" => "Sistema de abastecimiento de agua contra incendios",
        "HID" => "Hidrantes",
        "CS" => "Columna seca",
        "SCH" => "Sistema de control de humos y de calor",
        "SEN" => "Señalización luminiscente",
        "SFE" => "Sistemas fijos de extinción",
        _ => "Otros sistemas de protección contra incendios",
    }
}

/// Prepara los datos de la plantilla de contrato.
pub fn datos_contrato(b: &Biblioteca, k: &Contrato, v: &Valoracion, partes: &Value) -> Value {
    let desc = |codigo: &str, d: &str| {
        if d.is_empty() {
            b.catalogo
                .get(codigo)
                .map(|e| e.descripcion.clone())
                .unwrap_or_else(|| codigo.into())
        } else {
            d.to_string()
        }
    };

    // RITE
    let mut rite = Vec::new();
    for (ins, reg) in k
        .instalaciones
        .iter()
        .zip(&v.regimenes)
        .filter(|(i, _)| i.familia == "RITE")
    {
        let equipos: Vec<Value> = ins
            .equipos
            .iter()
            .map(|l| json!([desc(&l.codigo, &l.descripcion), l.cantidad]))
            .collect();
        let mut grupos: BTreeMap<(u32, String), Vec<String>> = BTreeMap::new();
        for lv in v
            .lineas
            .iter()
            .filter(|l| l.instalacion == ins.titulo && l.modo == "propio")
        {
            for o in &lv.operaciones {
                if o.veces_anio <= 0.0 {
                    continue;
                }
                let g = grupos
                    .entry(periodicidad_rite(&o.periodicidad, o.veces_anio))
                    .or_default();
                if !g.contains(&o.texto) {
                    g.push(o.texto.clone());
                }
            }
        }
        rite.push(json!({
            "titulo": if ins.titulo.is_empty() { "Instalaciones térmicas".to_string() } else { ins.titulo.clone() },
            "potencia_kw": ins.potencia_kw,
            "regimen": reg.descripcion,
            "mas_de_70": ins.potencia_kw.unwrap_or(0.0) > 70.0,
            "equipos": equipos,
            "grupos": grupos.into_iter().map(|((_, t), ops)| json!({"periodicidad": t, "operaciones": ops})).collect::<Vec<_>>(),
        }));
    }

    // Protección contra incendios y otros sistemas
    let mut sistemas: BTreeMap<String, (Vec<String>, Vec<Value>, bool)> = BTreeMap::new();
    let mut otros: BTreeMap<&str, Vec<Value>> = BTreeMap::new();
    for ins in k.instalaciones.iter().filter(|i| i.familia != "RITE") {
        for l in &ins.equipos {
            let Some(eq) = b.catalogo.get(&l.codigo) else { continue };
            let fila = json!([desc(&l.codigo, &l.descripcion), l.cantidad]);
            let modo = l.modo.clone().unwrap_or_else(|| eq.modo_defecto.clone());
            match eq.sistema_ripci.as_str() {
                "CO" => otros.entry("CO").or_default().push(fila),
                "AE" => otros.entry("AE").or_default().push(fila),
                "" => otros.entry("OTRO").or_default().push(fila),
                sub => {
                    let s = sistemas.entry(sistema_ripci(sub).to_string()).or_default();
                    if !s.0.contains(&sub.to_string()) {
                        s.0.push(sub.to_string());
                    }
                    s.1.push(fila);
                    s.2 |= modo == "subcontrata";
                }
            }
        }
    }
    let orden = ["EXT", "DET", "BIE", "HID", "CS", "ABAST", "SFE", "SCH", "SEN"];
    let mut claves: Vec<_> = sistemas.keys().cloned().collect();
    claves.sort_by_key(|c| orden.iter().position(|o| o == c).unwrap_or(99));
    let sistemas_json: Vec<Value> = claves
        .iter()
        .map(|c| {
            let (subs, equipos, sub) = &sistemas[c];
            let mut s = json!({
                "codigo": c, "subsistemas": subs, "titulo": titulo_sistema(c),
                "modo": if *sub { "subcontrata" } else { "propio" }, "equipos": equipos,
            });
            if c == "SCH" {
                s["norma_adicional"] = json!("Según la Guía técnica de aplicación del RIPCI, en emplazamientos subterráneos el mantenimiento se realizará conforme a la norma UNE 23584.");
            }
            s
        })
        .collect();
    let co_op = b
        .operaciones
        .get("CO-REV")
        .map(|o| o.texto.clone())
        .unwrap_or_else(|| "Comprobación de detectores y central de CO con gas de ensayo y de la puesta en marcha de la ventilación por niveles.".into());
    let mut otros_json = Vec::new();
    if let Some(eq) = otros.remove("CO") {
        otros_json.push(json!({
            "titulo": "Ventilación del garaje por monóxido de carbono (CTE DB HS 3)",
            "equipos": eq, "periodicidad": "En cada visita",
            "operaciones": [co_op,
                "Comprobación de la puesta en marcha de los ventiladores por niveles de CO y de la señalización de alarma.",
                "Revisión del cuadro eléctrico de ventilación: contactores, protecciones térmicas y maniobra, conforme al REBT."],
        }));
    }
    if let Some(eq) = otros.remove("AE") {
        otros_json.push(json!({
            "titulo": "Alumbrado de emergencia (REBT, ITC-BT-28)",
            "equipos": eq, "periodicidad": "Anual",
            "operaciones": ["Prueba de funcionamiento y de autonomía de las luminarias de emergencia.",
                "Comprobación del estado de las baterías, lámparas y difusores, y sustitución de las que no superen la prueba (material aparte)."],
        }));
    }
    if let Some(eq) = otros.remove("OTRO") {
        otros_json.push(json!({
            "titulo": "Otros equipos", "equipos": eq, "periodicidad": "En cada visita",
            "operaciones": ["Revisión según las instrucciones del fabricante."],
        }));
    }

    // Visitas, importe y pago
    let t = &v.totales;
    let periodo = periodo_pago(v.visitas);
    let descripcion_visitas = v
        .regimenes
        .iter()
        .map(|r| {
            format!(
                "{}: {}.",
                if r.instalacion.is_empty() {
                    "Instalación"
                } else {
                    &r.instalacion
                },
                r.descripcion
            )
        })
        .collect::<Vec<_>>()
        .join(" ");
    let p = |n: &str| b.parametro(k.anio, n).ok();
    let av = &v.avisos_averia;
    let partes_cond = partes.get("condiciones").cloned().unwrap_or(json!({}));
    let cond = |clave: &str, defecto: String| -> String {
        partes_cond
            .get(clave)
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or(defecto)
    };
    let hay = |f: &str| k.instalaciones.iter().any(|i| i.familia == f);
    let mut titulos = Vec::new();
    if hay("RITE") {
        titulos.push("instalaciones térmicas");
    }
    if !sistemas_json.is_empty() {
        titulos.push("protección contra incendios");
    }
    for o in &otros_json {
        let t = o["titulo"].as_str().unwrap_or("");
        if t.starts_with("Ventilación") {
            titulos.push("ventilación del garaje");
        } else if t.starts_with("Alumbrado") {
            titulos.push("alumbrado de emergencia");
        } else {
            titulos.push("otros equipos");
        }
    }
    let subtitulo = {
        let s = match titulos.len() {
            0 => String::new(),
            1 => titulos[0].to_string(),
            n => format!("{} y {}", titulos[..n - 1].join(", "), titulos[n - 1]),
        };
        let mut c = s.chars();
        c.next()
            .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
            .unwrap_or_default()
    };

    let mut d = partes.clone();
    if !d.is_object() {
        d = json!({});
    }
    let o = d.as_object_mut().expect("objeto");
    let contrato = o.entry("contrato").or_insert_with(|| json!({}));
    if contrato.get("titulo").is_none() {
        contrato["titulo"] = json!(format!("Contrato de mantenimiento de {}", subtitulo.to_lowercase()));
    }
    o.insert("subtitulo".into(), json!(subtitulo));
    o.insert("rite".into(), json!(rite));
    o.insert("sistemas".into(), json!(sistemas_json));
    o.entry("excluir_operaciones").or_insert_with(|| json!([]));
    o.insert("otros_sistemas".into(), json!(otros_json));
    o.insert(
        "visitas".into(),
        json!({"anuales": v.visitas, "descripcion": descripcion_visitas}),
    );
    o.insert(
        "economia".into(),
        json!({
            "importe_anual": euros(t.precio_anual),
            "importe_anual_letra": euros_en_letras(t.precio_anual),
            "importe_visita": euros(t.precio_visita),
            "periodo_pago": periodo,
            "iva": "IVA no incluido",
        }),
    );
    o.insert(
        "avisos".into(),
        json!({
            "modalidad": av.modalidad,
            "incluidos": av.incluidos,
            "estimados": av.estimados,
            "precio_hora_oficial": p("precio_hora_oficial").map(euros),
            "precio_hora_pareja": p("precio_hora_pareja").map(euros),
            "precio_salida": p("precio_salida").map(euros),
        }),
    );
    o.insert(
        "condiciones".into(),
        json!({
            "duracion": cond("duracion", "un año desde la fecha de firma".into()),
            "preaviso": cond("preaviso", "un mes".into()),
            "revision_precios": cond("revision_precios", "En cada prórroga, el importe se actualizará con la variación interanual del Índice de Precios de Consumo (IPC general, INE) publicada para el mes anterior al vencimiento.".into()),
            "plazo_respuesta_averias": cond("plazo_respuesta_averias", "48 horas laborables desde la comunicación".into()),
            "plazo_urgencias": cond("plazo_urgencias", "4 horas para las averías que dejen sin servicio la instalación o afecten a la seguridad".into()),
            "forma_pago": cond("forma_pago", format!(
                "mediante facturación {periodo} de {} coincidiendo con cada una de las {} visitas, a 30 días fecha de factura",
                euros(t.precio_visita), v.visitas)),
        }),
    );
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letras_y_euros() {
        assert_eq!(euros(1234.5), "1.234,50 €");
        assert_eq!(euros(999.999), "1.000,00 €");
        assert_eq!(euros_en_letras(2000.0), "dos mil euros");
        assert_eq!(
            euros_en_letras(1521.30),
            "mil quinientos veintiún euros con treinta céntimos"
        );
        assert_eq!(en_letras(21_000), "veintiún mil");
        assert_eq!(en_letras(100), "cien");
        assert_eq!(en_letras(13_340), "trece mil trescientos cuarenta");
        assert_eq!(euros_en_letras(101.0), "ciento un euros");
        assert_eq!(periodo_pago(4), "trimestral");
        assert_eq!(periodo_pago(12), "mensual");
    }
}
