//! Carga de la biblioteca normativa y de los datos de empresa (CSV con separador `;`).

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::Error;

/// Equipo del catálogo (RITE o PCI).
#[derive(Debug, Clone, Deserialize)]
pub struct Equipo {
    /// Código del tipo de equipo en el catálogo.
    pub codigo: String,
    /// `RITE`, `PCI` u otra familia sin régimen normativo propio.
    pub familia: String,
    /// Descripción (marca y modelo si se conoce).
    pub descripcion: String,
    /// Unidad de medición.
    pub unidad: String,
    /// Código de sistema del anexo II del RIPCI (solo PCI).
    pub sistema_ripci: String,
    /// Fila de la tabla 3.1 del RITE que le corresponde (instalaciones ≤70 kW).
    pub equipo_tabla_3_1: String,
    /// `propio` o `subcontrata`.
    pub modo_defecto: String,
    /// Partida de la tarifa de subcontrata.
    pub partida_subcontrata: String,
}

/// Tiempo estándar de una operación sobre un tipo de equipo.
#[derive(Debug, Clone, Deserialize)]
pub struct Tiempo {
    /// Código del equipo en el catálogo.
    pub codigo_equipo: String,
    /// Identificador de la operación normativa.
    pub operacion_id: String,
    /// Minutos por unidad e intervención.
    pub minutos: f64,
    /// Técnicos necesarios (1 = oficial; 2 = oficial + ayudante).
    pub tecnicos: u32,
    /// Consumibles por unidad e intervención (€).
    pub consumibles_eur: f64,
    /// Estado del dato (`estimacion_borrador`, `revisado`…).
    pub estado: String,
    /// Observaciones.
    pub nota: String,
}

/// Operación normativa o de fabricante, con su periodicidad.
#[derive(Debug, Clone)]
pub struct Operacion {
    /// Identificador estable (no se renumera).
    pub id: String,
    /// Texto de la operación.
    pub texto: String,
    /// Código de periodicidad tal como figura en la fuente (m, t, 2t, 3m, 1a, visita…).
    pub periodicidad: String,
    /// `preventivo` entra en la cuota; el resto (5 años, vida útil…) se oferta aparte.
    pub tipo: String,
    /// Referencia normativa.
    pub referencia: String,
    /// Sistema RIPCI al que pertenece (vacío en RITE).
    pub sistema: String,
    /// Documento del que se ha transcrito la operación (trazabilidad).
    pub fuente: String,
    /// Estado de cotejo con la fuente oficial (`si`, `no`, `si_guia_v4`…).
    pub verificado: String,
}

/// Fila de la tabla 3.1 del RITE: meses entre revisiones según equipo, potencia y uso.
#[derive(Debug, Clone, Deserialize)]
pub struct FilaTabla31 {
    /// Identificador estable (no se renumera).
    pub id: String,
    /// Tipo de equipo tal como figura en la tabla 3.1.
    pub equipo: String,
    /// Potencia nominal mínima del tramo (kW), excluida.
    pub pn_min_kw: Option<f64>,
    /// Potencia nominal máxima del tramo (kW), incluida.
    pub pn_max_kw: Option<f64>,
    /// Meses entre revisiones en viviendas.
    pub viviendas_meses: f64,
    /// Meses entre revisiones en otros usos.
    pub otros_usos_meses: f64,
}

/// Partida de la tarifa de un subcontratista.
#[derive(Debug, Clone, Deserialize)]
pub struct PartidaSubcontrata {
    /// Año de los parámetros y tarifas aplicados.
    pub anio: u32,
    /// Subcontratista.
    pub proveedor: String,
    /// Identificador de la partida en la tarifa.
    pub partida: String,
    /// Descripción de la partida en la tarifa del proveedor.
    pub elemento: String,
    /// Unidad de medición.
    pub unidad: String,
    /// Precio por unidad y revisión (€); vacío si es «a consultar».
    pub precio: Option<f64>,
    /// Tipo: `preventivo`, `periodico_largo`, `incluido`, `aparte`…
    pub tipo: String,
}

/// Todo lo necesario para valorar: normativa + datos de empresa.
#[derive(Debug, Clone, Default)]
pub struct Biblioteca {
    /// Catálogo de equipos por código.
    pub catalogo: HashMap<String, Equipo>,
    /// Tiempos estándar.
    pub tiempos: Vec<Tiempo>,
    /// Operaciones normativas y de fabricante por id.
    pub operaciones: HashMap<String, Operacion>,
    /// Tabla 3.1 del RITE.
    pub tabla_3_1: Vec<FilaTabla31>,
    /// Tarifas de subcontrata.
    pub subcontrata: Vec<PartidaSubcontrata>,
    /// (año, parámetro) → valor
    pub parametros: HashMap<(u32, String), f64>,
}

fn lector(path: &Path) -> Result<csv::Reader<std::fs::File>, Error> {
    let f = std::fs::File::open(path).map_err(|e| Error::Fichero(path.display().to_string(), e.to_string()))?;
    Ok(csv::ReaderBuilder::new().delimiter(b';').flexible(true).from_reader(f))
}

fn leer<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Vec<T>, Error> {
    let mut rd = lector(path)?;
    // Quita un posible BOM UTF-8 de la primera cabecera.
    let mut cab = rd
        .headers()
        .map_err(|e| Error::Csv(path.display().to_string(), e.to_string()))?
        .clone();
    if let Some(c) = cab.get(0) {
        if c.starts_with('\u{feff}') {
            let limpia: Vec<String> = cab
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    if i == 0 {
                        s.trim_start_matches('\u{feff}').to_string()
                    } else {
                        s.to_string()
                    }
                })
                .collect();
            cab = csv::StringRecord::from(limpia);
            rd.set_headers(cab);
        }
    }
    rd.deserialize()
        .map(|r| r.map_err(|e| Error::Csv(path.display().to_string(), e.to_string())))
        .collect()
}

#[derive(Deserialize)]
struct FilaRite33 {
    id: String,
    operacion: String,
    periodicidad: String,
    referencia: String,
    fuente: String,
    verificado: String,
}

#[derive(Deserialize)]
struct FilaRipci {
    id: String,
    tabla: String,
    sistema_codigo: String,
    periodicidad: String,
    operacion: String,
    tipo: String,
    fuente: String,
    verificado: String,
}

#[derive(Deserialize)]
struct FilaFabricante {
    id: String,
    periodicidad: String,
    operacion: String,
}

#[derive(Deserialize)]
struct FilaParametro {
    anio: u32,
    parametro: String,
    valor: f64,
}

#[derive(Deserialize)]
struct FilaT31Raw {
    id: String,
    equipo: String,
    pn_min_kw: String,
    pn_max_kw: String,
    viviendas_meses: f64,
    otros_usos_meses: f64,
}

#[derive(Deserialize)]
struct FilaSubRaw {
    anio: u32,
    proveedor: String,
    partida: String,
    elemento: String,
    unidad: String,
    precio: String,
    tipo: String,
}

fn num(s: &str) -> Option<f64> {
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        t.replace(',', ".").parse().ok()
    }
}

impl Biblioteca {
    /// Carga la biblioteca desde los CSV de semilla.
    ///
    /// * `normativa`: carpeta con las tablas del RITE, del RIPCI y de fabricante (pública).
    /// * `empresa`: carpeta con catálogo, tiempos, parámetros y tarifas de subcontrata
    ///   (en el repositorio solo hay un ejemplo ficticio; los datos reales van en `privado/`).
    ///
    /// Solo se usa para crear la base de datos (`db::inicializar`) y en las pruebas.
    pub fn desde_csv(normativa: &Path, empresa: &Path) -> Result<Self, Error> {
        let bib = normativa;
        let emp = empresa;
        let mut b = Biblioteca::default();

        for e in leer::<Equipo>(&emp.join("catalogo_equipos.csv"))? {
            b.catalogo.insert(e.codigo.clone(), e);
        }
        b.tiempos = leer(&emp.join("tiempos_estandar.csv"))?;

        for r in leer::<FilaRite33>(&bib.join("rite_tabla_3_3_operaciones.csv"))? {
            b.operaciones.insert(
                r.id.clone(),
                Operacion {
                    id: r.id,
                    texto: r.operacion,
                    periodicidad: r.periodicidad,
                    tipo: "preventivo".into(),
                    referencia: r.referencia,
                    sistema: String::new(),
                    fuente: r.fuente,
                    verificado: r.verificado,
                },
            );
        }
        for r in leer::<FilaRipci>(&bib.join("ripci_anexo_II_operaciones.csv"))? {
            b.operaciones.insert(
                r.id.clone(),
                Operacion {
                    id: r.id,
                    texto: r.operacion,
                    periodicidad: r.periodicidad,
                    tipo: r.tipo,
                    referencia: format!("RIPCI anexo II, tabla {}", r.tabla),
                    sistema: r.sistema_codigo,
                    fuente: r.fuente,
                    verificado: r.verificado,
                },
            );
        }
        for r in leer::<FilaFabricante>(&bib.join("operaciones_fabricante.csv"))? {
            b.operaciones.insert(
                r.id.clone(),
                Operacion {
                    id: r.id,
                    texto: r.operacion,
                    periodicidad: r.periodicidad,
                    tipo: "preventivo".into(),
                    referencia: "Fabricante".into(),
                    sistema: String::new(),
                    fuente: "Instrucciones del fabricante".into(),
                    verificado: "no".into(),
                },
            );
        }
        for r in leer::<FilaT31Raw>(&bib.join("rite_tabla_3_1_frecuencia_revisiones.csv"))? {
            b.tabla_3_1.push(FilaTabla31 {
                id: r.id,
                equipo: r.equipo,
                pn_min_kw: num(&r.pn_min_kw),
                pn_max_kw: num(&r.pn_max_kw),
                viviendas_meses: r.viviendas_meses,
                otros_usos_meses: r.otros_usos_meses,
            });
        }
        for r in leer::<FilaSubRaw>(&emp.join("tarifa_subcontrata.csv"))? {
            b.subcontrata.push(PartidaSubcontrata {
                anio: r.anio,
                proveedor: r.proveedor,
                partida: r.partida,
                elemento: r.elemento,
                unidad: r.unidad,
                precio: num(&r.precio),
                tipo: r.tipo,
            });
        }
        for r in leer::<FilaParametro>(&emp.join("parametros.csv"))? {
            b.parametros.insert((r.anio, r.parametro), r.valor);
        }
        Ok(b)
    }

    /// Parámetro del año indicado; si no existe, el del año anterior más próximo.
    pub fn parametro(&self, anio: u32, nombre: &str) -> Result<f64, Error> {
        (2000..=anio)
            .rev()
            .find_map(|a| self.parametros.get(&(a, nombre.to_string())).copied())
            .ok_or_else(|| Error::Parametro(nombre.to_string(), anio))
    }

    /// Tiempos estándar de un tipo de equipo.
    pub fn tiempos_de(&self, codigo: &str) -> impl Iterator<Item = &Tiempo> {
        let c = codigo.to_string();
        self.tiempos.iter().filter(move |t| t.codigo_equipo == c)
    }

    /// Partida de subcontrata vigente para el año (la más reciente ≤ año).
    pub fn partida(&self, id: &str, anio: u32) -> Option<&PartidaSubcontrata> {
        self.subcontrata
            .iter()
            .filter(|p| p.partida == id && p.anio <= anio)
            .max_by_key(|p| p.anio)
    }

    /// Meses entre revisiones según la tabla 3.1 (instalaciones ≤70 kW).
    pub fn meses_tabla_3_1(&self, equipo_t31: &str, potencia_kw: f64, viviendas: bool) -> Option<f64> {
        self.tabla_3_1
            .iter()
            .filter(|f| f.equipo == equipo_t31)
            .find(|f| f.pn_min_kw.is_none_or(|m| potencia_kw > m) && f.pn_max_kw.is_none_or(|m| potencia_kw <= m))
            .map(|f| {
                if viviendas {
                    f.viviendas_meses
                } else {
                    f.otros_usos_meses
                }
            })
    }
}
