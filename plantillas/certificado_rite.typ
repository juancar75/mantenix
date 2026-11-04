// Certificado anual de mantenimiento de instalaciones térmicas (RITE, art. 28).
//
// Contenido mínimo del art. 28.2: identificación de la instalación y de la empresa mantenedora,
// declaración de mantenimiento conforme al Manual de Uso y Mantenimiento y a la IT 3, y
// resumen anual de consumos y aportaciones. Si la comunidad autónoma tiene modelo oficial, se
// presenta en ese modelo y este documento sirve de anexo con el detalle.
//
// Entradas (sys.inputs, texto JSON):
//   cert     → {anio, periodo, instalaciones: [{titulo, potencia_kw, equipos: [[desc, n]]}],
//               visitas, director, consumos: {...}, aportaciones: {...}}
//   datos    → partes del contrato (cliente, edificio, contrato)
//   empresa  → datos de la empresa (sin logos)
//   membrete → {habilitaciones, logo} del ámbito RITE
#import "comun.typ": *

#let cert = json(bytes(sys.inputs.cert))
#let d = json(bytes(sys.inputs.datos))
#let empresa = json(bytes(sys.inputs.empresa))
#let membrete = json(bytes(sys.inputs.membrete))
#let c = d.at("cliente", default: (razon_social: "", cif: "", domicilio: ""))
#let e = d.at("edificio", default: (nombre: "", direccion: "", uso: ""))
#let ref = d.at("contrato", default: (:)).at("referencia", default: "")
#let razon = empresa.at("razon_social", default: "")
#let hueco(w: 100%) = box(width: w, stroke: (bottom: 0.5pt + gris), height: 1em)
#let dato(x) = if x == none or x == "" { hueco(w: 4cm) } else { x }
#let habil = membrete.habilitaciones.map(h => h.at(0) + ": " + h.at(1)).join(" · ")

#set document(title: "Certificado de mantenimiento " + str(cert.anio) + " · " + e.nombre)
#set text(font: "Inter", size: 9pt, lang: "es", region: "es")
#set par(justify: true, leading: 0.6em)
#show grid: set par(justify: false)
#set table(stroke: 0.4pt + linea, inset: (x: 5pt, y: 4pt))
#show table.cell.where(y: 0): set text(weight: "bold", size: 8pt, fill: azul)
#show heading.where(level: 1): it => block(above: 1.1em, below: 0.5em, sticky: true,
  text(10pt, weight: "bold", fill: azul, it.body))
#set page(paper: "a4", margin: (x: 2cm, top: 1.6cm, bottom: 1.6cm),
  footer: context { set text(7pt, fill: gris)
    [#razon · Certificado de mantenimiento #cert.anio · #e.nombre #h(1fr) Página #counter(page).display() de #counter(page).final().first()] })

#grid(columns: (auto, 1fr), column-gutter: 12pt, align: horizon,
  if membrete.logo != none { image(membrete.logo, height: 1.3cm) } else { text(11pt, weight: "bold", fill: azul, razon) },
  { set align(right); text(14pt, weight: "bold", fill: azul)[Certificado de mantenimiento]; linebreak()
    text(9pt, fill: gris)[Instalaciones térmicas · RITE (RD 1027/2007), artículo 28 · Año #cert.anio] })
#v(2pt)
#line(length: 100%, stroke: 0.7pt + azul)

= a) Identificación de la instalación
#grid(columns: (5.2cm, 1fr), row-gutter: 5pt,
  [Edificio], [*#e.nombre*],
  [Dirección], [#e.direccion],
  [Uso], [#e.at("uso", default: "")],
  [Titular], [#c.razon_social #if c.at("cif", default: "") != "" [(CIF #c.cif)]],
  [N.º de expediente inicial], [#dato(e.at("expediente", default: ""))],
  [Contrato de mantenimiento], [#ref],
  [Periodo certificado], [#cert.periodo],
)
#for i in cert.instalaciones [
  *#i.titulo* — potencia térmica nominal #num(i.potencia_kw, d: 0) kW
  #table(columns: (1fr, 2cm), align: (left, right),
    table.header[Equipo][Cantidad],
    ..i.equipos.map(q => (q.at(0), num(q.at(1), d: 0))).flatten())
]

= b) Empresa mantenedora
#grid(columns: (5.2cm, 1fr), row-gutter: 5pt,
  [Empresa mantenedora], [*#razon* #if empresa.at("cif", default: "") != "" [(CIF #empresa.cif)]],
  [Habilitación], [#habil],
  [Mantenedor habilitado responsable], [#hueco(w: 6cm) Carné n.º #hueco(w: 3cm)],
  [Director de mantenimiento], [#if cert.director [#hueco(w: 6cm) (preceptivo: más de 5.000 kW en calor o 1.000 kW en frío)] else [No preceptivo (art. 26.6)]],
)

= c) Declaración
La empresa mantenedora abajo firmante *declara* que, durante el periodo indicado, la instalación térmica descrita ha sido mantenida de acuerdo con su «Manual de Uso y Mantenimiento» y que cumple con los requisitos exigidos en la IT 3 del Reglamento de Instalaciones Térmicas en los Edificios. Se han realizado #cert.visitas visitas de mantenimiento preventivo, registradas en el registro de operaciones de mantenimiento de la instalación (art. 27), que forma parte del Libro del Edificio.

= d) Resumen de los consumos anuales registrados
#let cons = cert.at("consumos", default: (:))
#table(columns: (1fr, 3.5cm, 2.5cm), align: (left, right, left),
  table.header[Concepto][Consumo anual][Unidad],
  [Combustible], [#cons.at("combustible", default: "")], [#cons.at("combustible_ud", default: "")],
  [Energía eléctrica], [#cons.at("electricidad", default: "")], [kWh],
  [Agua para llenado de las instalaciones], [#cons.at("agua_llenado", default: "")], [m³],
  [Agua caliente sanitaria], [#cons.at("acs", default: "")], [m³],
  [Totalización de contadores individuales de ACS], [#cons.at("acs_individual", default: "")], [m³],
  [Energía térmica], [#cons.at("energia_termica", default: "")], [kWh],
)

= e) Resumen de las aportaciones anuales
#let ap = cert.at("aportaciones", default: (:))
#table(columns: (1fr, 3.5cm, 2.5cm), align: (left, right, left),
  table.header[Concepto][Aportación anual][Unidad],
  [Térmica de la central de producción], [#ap.at("central", default: "")], [kWh],
  [Energías renovables], [#ap.at("renovables", default: "")], [kWh],
  [Cogeneración], [#ap.at("cogeneracion", default: "")], [kWh],
)
#par(justify: false)[Si alguno de estos datos no puede obtenerse (por ejemplo, por falta de contadores), se justifica a continuación:] #hueco()

#v(8pt)
#grid(columns: (1fr, 1fr), column-gutter: 24pt,
  [En #hueco(w: 3cm), a #hueco(w: 3.5cm) \ \ *Por la empresa mantenedora* \ #v(1.4cm) Firma y sello],
  [#if cert.director [\ \ *El director de mantenimiento* \ #v(1.4cm) Firma] else [\ \ *Recibí del titular* \ #v(1.4cm) Firma]])
#v(6pt)
#text(7.5pt, fill: gris)[Validez máxima de un año (art. 28.1). Una copia queda en poder del titular, que la incorpora al Libro del Edificio; se envía al órgano competente de la comunidad autónoma cuando así se determine, en su modelo oficial si lo tiene.]
