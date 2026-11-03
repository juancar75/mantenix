//! Línea de comandos de Mantenix.
//!
//! Sirve para automatizar tareas y para probar el núcleo sin la interfaz gráfica.
//! La aplicación de escritorio (`app/`) usa exactamente las mismas funciones.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use mantenix_core::{db, tarifa, valorar, Biblioteca, Contrato};

#[derive(Parser)]
#[command(
    name = "mantenix",
    version,
    about = "Valoración de contratos de mantenimiento RITE/RIPCI"
)]
struct Cli {
    /// Base de datos SQLite.
    #[arg(long, default_value = "mantenix.db", global = true)]
    db: PathBuf,
    /// Usuario al que se atribuyen los cambios en el registro de auditoría.
    #[arg(long, global = true, env = "MANTENIX_USUARIO")]
    usuario: Option<String>,
    #[command(subcommand)]
    orden: Orden,
}

#[derive(Subcommand)]
enum Orden {
    /// Crea la base de datos con la normativa y los datos de empresa iniciales.
    Init {
        /// Carpeta con la normativa (RITE, RIPCI).
        #[arg(long, default_value = "datos/semilla/normativa")]
        normativa: PathBuf,
        /// Carpeta con los datos de empresa (catálogo, tiempos, parámetros, tarifas).
        /// Usar `privado/empresa` para los datos reales.
        #[arg(long, default_value = "datos/semilla/ejemplo")]
        empresa: PathBuf,
    },
    /// Da de alta (o actualiza) un contrato con su inventario desde JSON.
    Importar { inventario: PathBuf },
    /// Lista los contratos.
    Contratos,
    /// Carga los datos de la empresa (razón social, habilitaciones, horario) desde un JSON.
    Empresa { json: PathBuf },
    /// Fija un parámetro de un año (coste hora, margen…).
    Parametro { anio: u32, nombre: String, valor: f64 },
    /// Fija el tiempo estándar de una operación sobre un tipo de equipo.
    Tiempo {
        equipo: String,
        operacion: String,
        minutos: f64,
        #[arg(default_value_t = 1)]
        tecnicos: u32,
    },
    /// Muestra el registro de cambios.
    Historial {
        #[arg(long)]
        tabla: Option<String>,
        #[arg(long, default_value_t = 20)]
        limite: u32,
    },
    /// Valora un contrato y, opcionalmente, genera la hoja de costes.
    Valorar {
        nombre: String,
        #[arg(long)]
        pdf: Option<PathBuf>,
    },
    /// Calcula la tarifa propia del año.
    Tarifa {
        #[arg(long)]
        anio: u32,
        #[arg(long)]
        pdf: Option<PathBuf>,
    },
    /// Guarda un presupuesto desde JSON (sin número = el siguiente libre) y genera la carta en PDF.
    Presupuesto {
        json: PathBuf,
        #[arg(long)]
        pdf: Option<PathBuf>,
    },
    /// Lista los presupuestos (opcionalmente de un cliente) y genera la relación en PDF.
    Presupuestos {
        #[arg(long)]
        cliente: Option<String>,
        #[arg(long)]
        pdf: Option<PathBuf>,
    },
    /// Carga el logo (PNG, JPG o SVG) de un ámbito: RITE, PCI, REBT o «general».
    Logo { ambito: String, fichero: PathBuf },
    /// Genera el contrato en PDF de un contrato guardado: inventario y precio de su valoración,
    /// y los datos de las partes (cliente, edificio, referencia, condiciones) de un JSON.
    ContratoPdf {
        nombre: String,
        /// JSON con las partes; si falta, se usan las guardadas con el contrato.
        #[arg(long)]
        partes: Option<PathBuf>,
        #[arg(long)]
        pdf: PathBuf,
    },
    /// Genera el PDF de un contrato desde su JSON de datos (la empresa y las operaciones salen de la base).
    Contrato {
        datos: PathBuf,
        #[arg(long)]
        pdf: PathBuf,
    },
}

fn leer_json(p: &PathBuf) -> Result<serde_json::Value> {
    Ok(serde_json::from_str(
        &std::fs::read_to_string(p).with_context(|| format!("leyendo {}", p.display()))?,
    )?)
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut c = db::abrir(&cli.db)?;
    if let Some(u) = &cli.usuario {
        db::fijar_usuario(&c, u)?;
    }
    match cli.orden {
        Orden::Init { normativa, empresa } => {
            if let Some(semilla) = normativa.parent() {
                let n = db::cargar_modelos_semilla(&c, &semilla.join("presupuestos/modelos.json"))?;
                if n > 0 {
                    println!("{n} modelos de presupuesto cargados.");
                }
            }
            if db::inicializar(&mut c, &normativa, &empresa)? {
                println!("Base creada en {} (empresa: {}).", cli.db.display(), empresa.display());
            } else {
                println!("La base {} ya tenía datos; no se toca.", cli.db.display());
            }
        }
        Orden::Importar { inventario } => {
            let k: Contrato = serde_json::from_value(leer_json(&inventario)?)?;
            let id = db::guardar_contrato(&mut c, &k)?;
            println!("Contrato «{}» guardado ({id}).", k.nombre);
        }
        Orden::Empresa { json } => {
            db::fijar_empresa(&c, &leer_json(&json)?)?;
            println!("Datos de empresa actualizados.");
        }
        Orden::Contratos => {
            for (n, a, p) in db::listar_contratos(&c)? {
                println!(
                    "{n:<50} {a}  {}",
                    p.map(|x| format!("{x:.2} €")).unwrap_or_else(|| "sin valorar".into())
                );
            }
        }
        Orden::Parametro { anio, nombre, valor } => {
            db::fijar_parametro(&c, anio, &nombre, valor)?;
            println!("{anio} · {nombre} = {valor}");
        }
        Orden::Tiempo {
            equipo,
            operacion,
            minutos,
            tecnicos,
        } => {
            db::fijar_tiempo(&c, &equipo, &operacion, minutos, tecnicos)?;
            println!("{equipo} · {operacion} = {minutos} min × {tecnicos}");
        }
        Orden::Historial { tabla, limite } => {
            for h in db::historial(&c, tabla.as_deref(), limite)? {
                println!(
                    "#{} {} {:<10} {:<18} {:<6} {} → {}",
                    h.id,
                    h.fecha,
                    h.usuario.unwrap_or_default(),
                    h.tabla,
                    h.accion,
                    h.antes.unwrap_or_default(),
                    h.despues.unwrap_or_default()
                );
            }
        }
        Orden::Valorar { nombre, pdf } => {
            let b = Biblioteca::desde_db(&c)?;
            let (id, k) = db::cargar_contrato(&c, &nombre)?;
            let v = valorar(&b, &k)?;
            db::guardar_valoracion(&c, &id, &v)?;
            imprimir_resumen(&v);
            if let Some(pdf) = pdf {
                std::fs::write(&pdf, mantenix_informes::hoja_costes(&v, &db::empresa(&c)?)?)?;
                println!("PDF: {}", pdf.display());
            }
        }
        Orden::Tarifa { anio, pdf } => {
            let t = tarifa(&Biblioteca::desde_db(&c)?, anio)?;
            println!("Tarifa {anio}: {} partidas", t.filas.len());
            if let Some(pdf) = pdf {
                std::fs::write(&pdf, mantenix_informes::tarifa(&t, &db::empresa(&c)?)?)?;
                println!("PDF: {}", pdf.display());
            }
        }
        Orden::Presupuesto { json, pdf } => {
            let p: mantenix_core::Presupuesto = serde_json::from_value(leer_json(&json)?)?;
            let numero = db::guardar_presupuesto(&c, &p)?;
            let p = db::cargar_presupuesto(&c, &numero)?;
            let t = mantenix_core::presupuesto::calcular(&p);
            println!(
                "Presupuesto {numero}: base {:.2} €, total {:.2} € (margen {:.2} € · {:.1} %)",
                t.base, t.total, t.margen, t.margen_pct
            );
            if let Some(pdf) = pdf {
                std::fs::write(&pdf, mantenix_informes::presupuesto(&p, &db::empresa(&c)?)?)?;
                println!("PDF: {}", pdf.display());
            }
        }
        Orden::Logo { ambito, fichero } => {
            let ambito = if ambito.eq_ignore_ascii_case("general") {
                String::new()
            } else {
                ambito.to_uppercase()
            };
            let mut e = db::empresa(&c)?;
            mantenix_core::empresa::fijar_logo(&mut e, &ambito, Some(&std::fs::read(&fichero)?))?;
            db::fijar_empresa(&c, &e)?;
            println!("Logo cargado.");
        }
        Orden::Presupuestos { cliente, pdf } => {
            let lista: Vec<_> = db::listar_presupuestos(&c)?
                .into_iter()
                .filter(|r| cliente.as_ref().is_none_or(|x| &r.cliente == x))
                .collect();
            if let Some(pdf) = pdf {
                let titulo = cliente
                    .as_ref()
                    .map(|x| format!("Presupuestos · {x}"))
                    .unwrap_or_else(|| "Relación de presupuestos".into());
                let hoy: String = c.query_row("SELECT date('now','localtime')", [], |r| r.get(0))?;
                let bytes = mantenix_informes::relacion_presupuestos(&lista, &titulo, &hoy, &db::empresa(&c)?)?;
                std::fs::write(&pdf, bytes)?;
                println!("PDF: {}", pdf.display());
            }
            for r in &lista {
                println!(
                    "{}  {}  {:<10} {:>10.2} €  {} · {}",
                    r.numero, r.fecha, r.estado, r.total, r.cliente, r.titulo
                );
            }
        }
        Orden::ContratoPdf { nombre, partes, pdf } => {
            let b = Biblioteca::desde_db(&c)?;
            let (id, k) = db::cargar_contrato(&c, &nombre)?;
            let v = valorar(&b, &k)?;
            db::guardar_valoracion(&c, &id, &v)?;
            imprimir_resumen(&v);
            let partes = match partes {
                Some(p) => leer_json(&p)?,
                None => k.partes.clone(),
            };
            let datos = mantenix_core::documento::datos_contrato(&b, &k, &v, &partes);
            let bytes = mantenix_informes::contrato(&datos, &db::empresa(&c)?, b.operaciones.values())?;
            std::fs::write(&pdf, bytes)?;
            println!("PDF: {}", pdf.display());
        }
        Orden::Contrato { datos, pdf } => {
            let b = Biblioteca::desde_db(&c)?;
            let bytes = mantenix_informes::contrato(&leer_json(&datos)?, &db::empresa(&c)?, b.operaciones.values())?;
            std::fs::write(&pdf, bytes)?;
            println!("PDF: {}", pdf.display());
        }
    }
    Ok(())
}

fn eur(x: f64) -> String {
    format!("{x:>10.2} €")
}

fn imprimir_resumen(v: &mantenix_core::Valoracion) {
    let t = &v.totales;
    println!("\n{} — año {}", v.nombre, v.anio);
    for r in &v.regimenes {
        println!("  · {} → mín. {} visitas", r.descripcion, r.visitas_minimas);
    }
    println!("  Visitas: {} (mínimo normativo {})", v.visitas, v.visitas_minimas);
    println!(
        "  Horas equipos {:.1} h · desplazamiento/fijo {:.1} h",
        t.horas_equipos, t.horas_desplazamiento
    );
    println!("  Mano de obra   {}", eur(t.mano_obra));
    println!("  Consumibles    {}", eur(t.consumibles));
    println!("  Desplazamiento {}", eur(t.desplazamiento));
    println!(
        "  Subcontrata    {}  ({:.2} €/visita × {})",
        eur(t.subcontrata),
        v.subcontrata_visita_facturada,
        v.visitas_subcontrata
    );
    let a = &v.avisos_averia;
    println!(
        "  Avisos         {}  ({} incluidos, {:.1} estimados, {:.2} €/aviso, modalidad {})",
        eur(t.avisos),
        a.incluidos,
        a.estimados,
        a.coste_aviso,
        a.modalidad
    );
    println!("  Coste total    {}", eur(t.coste_total));
    println!(
        "  PRECIO ANUAL   {}  ({:.2} €/visita, margen {} %)",
        eur(t.precio_anual),
        t.precio_visita,
        t.margen_pct
    );
    if let (Some(pa), Some(m)) = (t.precio_actual_anual, t.margen_precio_actual_pct) {
        println!("  Precio actual  {}  → margen real {:.1} %", eur(pa), m);
    }
    for a in &v.avisos {
        println!("  ⚠ {a}");
    }
}
