// Carta presupuesto para el cliente: reparación (correctivo) o pequeña obra.
// Explica qué se ha roto o desgastado, sus consecuencias y el repuesto o trabajo propuesto,
// con el importe desglosado y un espacio para la aceptación. No muestra costes internos.
#import "comun.typ": *

// Entradas: datos = {p: Presupuesto, t: TotalesPresupuesto}, empresa = JSON de la empresa.
#let empresa = json(bytes(sys.inputs.empresa))
// membrete = {habilitaciones: [[texto, número]], logo: "logo.png" | none}, según el ámbito.
#let membrete = json(bytes(sys.inputs.membrete))
#let d = json(bytes(sys.inputs.datos))
#let p = d.p
#let t = d.t

#let meses = ("enero", "febrero", "marzo", "abril", "mayo", "junio", "julio", "agosto",
  "septiembre", "octubre", "noviembre", "diciembre")
#let fecha_dt(s) = datetime(year: int(s.slice(0, 4)), month: int(s.slice(5, 7)), day: int(s.slice(8, 10)))
#let larga(dt) = str(dt.day()) + " de " + meses.at(dt.month() - 1) + " de " + str(dt.year())
#let emision = fecha_dt(p.fecha)
#let valida_hasta = emision + duration(days: p.validez_dias)
#let campo(x) = x != none and x.trim() != ""
#let texto(x) = for (i, parrafo) in x.split("\n\n").enumerate() {
  if i > 0 { parbreak() }
  parrafo.split("\n").join(linebreak())
}

#let razon = empresa.at("razon_social", default: "")
#set document(title: "Presupuesto " + p.numero + " · " + p.titulo)
#set text(font: "Inter", size: 9pt, lang: "es", region: "es")
#set par(justify: true, leading: 0.6em, spacing: 0.8em)
#set table(stroke: (x, y) => (bottom: 0.5pt + linea), inset: (x: 5pt, y: 4.5pt))
#show table.cell.where(y: 0): set text(weight: "bold", size: 8pt, fill: azul)
#show heading.where(level: 1): it => block(above: 1.05em, below: 0.5em, sticky: true,
  text(10pt, weight: "bold", fill: azul, it.body))
#set page(paper: "a4", margin: (x: 2cm, top: 1.5cm, bottom: 1.6cm),
  footer: context {
    set text(7.5pt, fill: gris)
    [#razon #if campo(empresa.at("cif", default: "")) [· CIF #empresa.cif] #h(1fr)
     Presupuesto #p.numero · página #counter(page).display() de #counter(page).final().first()]
  })

// Membrete y destinatario
#grid(columns: (1fr, 1fr), column-gutter: 1.5cm,
  {
    set par(justify: false, leading: 0.45em)
    if membrete.logo != none {
      block(below: 0.5em, image(membrete.logo, height: 1.5cm, width: auto))
    }
    text(13pt, weight: "bold", fill: azul, razon)
    set text(7.5pt, fill: gris)
    linebreak()
    for k in ("domicilio", "telefono", "email") {
      if campo(empresa.at(k, default: "")) [#empresa.at(k) \ ]
    }
    for (nombre, numero) in membrete.habilitaciones [#nombre: *#numero* \ ]
  },
  block(inset: (left: 10pt), stroke: (left: 1.5pt + azul), {
    text(8pt, fill: gris, "Destinatario")
    linebreak()
    text(weight: "bold", p.cliente)
    if campo(p.cif) [ \ CIF/NIF: #p.cif]
    if campo(p.contacto) [ \ A/A: #p.contacto]
    if campo(p.direccion) [ \ #p.direccion]
  }),
)

#v(0.9em)
#grid(columns: (1fr, auto),
  [*Presupuesto nº #p.numero*],
  [#larga(emision)],
)
#v(0.2em)
#block(width: 100%, inset: 8pt, radius: 3pt, fill: fondo,
  [*Asunto:* #p.titulo#if campo(p.contrato) [ \ #text(8.5pt, fill: gris)[Contrato de mantenimiento: #p.contrato]]])

#v(0.8em)
Muy señores nuestros:

#if p.tipo == "correctivo" [
  #if campo(p.deteccion) [Con motivo de la #p.deteccion, ] else [Durante los trabajos de mantenimiento, ]
  nuestros técnicos han detectado una incidencia
  #if campo(p.equipo) [en *#p.equipo*]
  que les describimos a continuación, junto con la solución que les proponemos.
] else [
  Atendiendo a su solicitud, les presentamos nuestro presupuesto para
  #if campo(p.equipo) [*#p.equipo*] else [los trabajos indicados],
  con la descripción de la situación actual y la solución propuesta.
]

#if campo(p.averia) [
  = #if p.tipo == "correctivo" [Incidencia detectada] else [Situación actual]
  #texto(p.averia)
]

#if campo(p.consecuencias) [
  = Consecuencias de no actuar
  #texto(p.consecuencias)
]

#if campo(p.propuesta) [
  = Solución propuesta
  #texto(p.propuesta)
]

= Importe
#let filas = p.lineas.zip(t.lineas)
#table(columns: (1fr, auto, auto, auto, auto), align: (left, right, left, right, right),
  table.header[Concepto][Cantidad][Ud.][Precio][Importe],
  ..filas.map(((l, i)) => (
    l.descripcion,
    num(l.cantidad, d: if calc.fract(l.cantidad) == 0 { 0 } else { 2 }),
    l.unidad, eur(l.precio_unitario), eur(i.importe),
  )).flatten(),
)
#align(right, grid(columns: (auto, 3.2cm), align: (left, right), inset: (x: 5pt, y: 3pt),
  ..if t.descuento > 0 {
    ([Suma], eur(t.subtotal), [Descuento (#num(p.descuento_pct, d: 0) %)], [− #eur(t.descuento)])
  } else { () },
  [Base imponible], eur(t.base),
  [IVA (#num(p.iva_pct, d: 0) %)], eur(t.iva),
  grid.hline(stroke: 0.8pt + azul),
  text(10.5pt, weight: "bold", fill: azul)[TOTAL], text(10.5pt, weight: "bold", fill: azul, eur(t.total)),
))

= Condiciones
#if campo(p.condiciones) [
  #texto(p.condiciones)
] else [
  - Validez de la oferta: #p.validez_dias días (hasta el #larga(valida_hasta)).
  - Plazo de ejecución: a convenir tras la aceptación, según disponibilidad del repuesto.
  - Forma de pago: a la finalización de los trabajos, mediante transferencia bancaria.
  - Garantía: la del fabricante para el repuesto y tres meses para la mano de obra.
  - No incluye trabajos distintos de los descritos ni ayudas de albañilería, salvo indicación expresa.
]

#v(0.4em)
Quedamos a su disposición para cualquier aclaración. Atentamente,

#v(1.6em)
#let firmante = empresa.at("firmante", default: (:))
#if campo(firmante.at("nombre", default: "")) [*#firmante.nombre* \ #firmante.at("cargo", default: "") \ ]
#razon

#v(1fr)
#block(width: 100%, inset: 9pt, radius: 3pt, stroke: 0.5pt + linea, breakable: false, {
  text(weight: "bold", fill: azul)[Aceptación del presupuesto nº #p.numero]
  h(6pt)
  text(8pt, fill: gris)[Conforme con el presupuesto y sus condiciones. Rogamos nos devuelvan una copia firmada para programar los trabajos.]
  v(2.4em)
  grid(columns: (1fr, 1fr, 1fr), column-gutter: 14pt, row-gutter: 4pt,
    line(length: 100%, stroke: 0.5pt + gris), line(length: 100%, stroke: 0.5pt + gris),
    line(length: 100%, stroke: 0.5pt + gris),
    text(7.5pt, fill: gris)[Nombre y cargo], text(7.5pt, fill: gris)[Fecha], text(7.5pt, fill: gris)[Firma y sello],
  )
})
