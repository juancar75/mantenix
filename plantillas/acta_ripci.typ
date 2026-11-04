// Acta de mantenimiento de instalaciones de protección contra incendios (RIPCI).
//
// RD 513/2017, art. 17.e y 21.2, y anexo II, punto 5: certificado con la información general
// (a.1 a a.9), firmado por el responsable técnico de la empresa mantenedora y por el titular, y
// listas de comprobación por producto o sistema (b.1 a b.3), firmadas por quien hizo las
// operaciones. Formato equivalente a la serie UNE 23580. Se conserva al menos cinco años.
//
// Entradas (sys.inputs, texto JSON):
//   acta     → mantenix_core::documento::datos_acta_ripci
//   datos    → partes del contrato (cliente, edificio, contrato)
//   empresa  → datos de la empresa (sin logos)
//   membrete → {habilitaciones, logo} del ámbito PCI
#import "comun.typ": *

#let acta = json(bytes(sys.inputs.acta))
#let d = json(bytes(sys.inputs.datos))
#let d = if d == none { (:) } else { d }
#let empresa = json(bytes(sys.inputs.empresa))
#let membrete = json(bytes(sys.inputs.membrete))
#let c = d.at("cliente", default: (razon_social: "", domicilio: ""))
#let firm = c.at("firmante", default: (nombre: "", cargo: ""))
#let e = d.at("edificio", default: (nombre: "", direccion: ""))
#let ref = d.at("contrato", default: (:)).at("referencia", default: "")
#let razon = empresa.at("razon_social", default: "")
#let hueco(w: 100%) = box(width: w, stroke: (bottom: 0.5pt + gris), height: 1em)
#let o_hueco(x, w: 5cm) = if x == none or x == "" { hueco(w: w) } else { x }
#let caja = box(width: 8pt, height: 8pt, stroke: 0.6pt + gris, radius: 1pt)
#let habil = membrete.habilitaciones.map(h => h.at(1)).join(", ")

#set document(title: "Acta de mantenimiento " + acta.tipo_texto + " PCI · " + e.nombre)
#set text(font: "Inter", size: 8.5pt, lang: "es", region: "es")
#set par(justify: false, leading: 0.55em)
#set table(stroke: 0.4pt + linea, inset: (x: 4pt, y: 3.5pt))
#show table.cell.where(y: 0): set text(weight: "bold", size: 7.5pt, fill: azul)
#show heading.where(level: 1): it => block(above: 1em, below: 0.45em, sticky: true,
  text(10pt, weight: "bold", fill: azul, it.body))
#set page(paper: "a4", margin: (x: 1.6cm, top: 1.4cm, bottom: 1.5cm),
  footer: context { set text(7pt, fill: gris)
    [#razon · Acta de mantenimiento PCI · #e.nombre · Conservar 5 años (RIPCI, art. 21.2) #h(1fr) Página #counter(page).display() de #counter(page).final().first()] })

#let cabecera(titulo) = {
  grid(columns: (auto, 1fr), column-gutter: 12pt, align: horizon,
    if membrete.logo != none { image(membrete.logo, height: 1.2cm) } else { text(11pt, weight: "bold", fill: azul, razon) },
    { set align(right); text(13pt, weight: "bold", fill: azul, titulo); linebreak()
      text(8pt, fill: gris)[Mantenimiento #acta.tipo_texto · RD 513/2017, anexo II · Contrato #ref] })
  v(2pt); line(length: 100%, stroke: 0.7pt + azul); v(2pt)
}

// ---------------- Certificado (información general) ----------------
#cabecera("Certificado de mantenimiento de protección contra incendios")
#grid(columns: (1fr, 1fr, 1fr), column-gutter: 10pt, [Fecha: #hueco(w: 2.6cm)], [Hora de inicio: #hueco(w: 1.6cm)], [Hora de fin: #hueco(w: 1.6cm)])

= a) Información general
#grid(columns: (6.2cm, 1fr), row-gutter: 6pt,
  [1.º Propiedad de la instalación], [*#c.razon_social* — #c.at("domicilio", default: "")],
  [2.º Representante de la propiedad responsable de la instalación], [#o_hueco(firm.at("nombre", default: "")) · #o_hueco(firm.at("cargo", default: ""), w: 3cm)],
  [3.º Representante de la propiedad ante estas operaciones], [#hueco(w: 6cm) Cargo: #hueco(w: 3cm)],
  [4.º Localización de la instalación y fecha de instalación], [#e.nombre, #e.direccion · Instalada: #o_hueco(e.at("fecha_instalacion", default: ""), w: 2.5cm)],
  [5.º Última inspección periódica (empresa y fecha)], [#hueco(w: 6cm) #o_hueco(acta.ultima_inspeccion, w: 2.5cm)],
  [6.º Último mantenimiento (empresa y fecha)], [#hueco(w: 6cm) #hueco(w: 2.5cm)],
  [7.º Empresa mantenedora], [*#razon* · N.º de habilitación #habil · #empresa.at("direccion", default: "") \ Declara que está habilitada para todos y cada uno de los productos y sistemas sobre los que efectúa este mantenimiento.],
  [8.º Persona/s responsable/s de las operaciones], [#hueco(w: 9cm) \ Declara/n estar cualificada/s para realizar estos mantenimientos.],
  [9.º Productos y sistemas objeto del mantenimiento], [#acta.sistemas.map(s => s.titulo).join(", ").],
)

#if acta.sistemas.len() == 0 [
  #text(fill: rgb("#a33"))[No hay sistemas de protección contra incendios mantenidos con medios propios en este contrato: el acta la emite la empresa que hace el mantenimiento.]
]

#v(6pt)
Se adjuntan las listas de comprobación de cada sistema, con las operaciones realizadas, sus resultados y las incidencias y acciones propuestas.
#v(8pt)
#grid(columns: (1fr, 1fr), column-gutter: 24pt,
  [*Responsable técnico de la empresa mantenedora* \ Nombre: \ #v(1.3cm) Firma y sello],
  [*Representante de la propiedad* (conformidad y recepción) \ Nombre: \ #v(1.3cm) Firma])

// ---------------- Listas de comprobación ----------------
#for s in acta.sistemas {
  pagebreak()
  cabecera("Lista de comprobación · " + s.titulo)
  [= b.1 / b.2) Productos o sistemas]
  table(columns: (1fr, 1.4cm, 4cm, 4.6cm), align: (left, right, left, left),
    table.header([Tipo], [Cantidad], [Marca y modelo], [Identificación (n.º de serie, ubicación)]),
    ..s.productos.map(p => (p.descripcion, num(p.cantidad, d: 0), v(1em), [])).flatten())
  if s.codigo == "EXT" or s.codigo == "DET" {
    text(7.5pt, fill: gris)[Si hay muchas unidades, adjunte la relación con su número de serie o de lazo y su ubicación.]
  }
  [= b.3) Operaciones realizadas y resultado]
  text(7.5pt, fill: gris)[*C* correcto · *I* incorrecto (describa la incidencia y la acción propuesta abajo) · *NA* no aplica. Periodicidad según las tablas I, II y III del anexo II.]
  table(columns: (1fr, 1cm, auto, auto, auto), align: (left, center, center, center, center),
    table.header([Operación], [Per.], [C], [I], [NA]),
    ..s.operaciones.map(o => (o.texto, text(7pt, o.periodicidad), caja, caja, caja)).flatten())
  [= Incidencias y acciones propuestas]
  table(columns: (1fr,), ..range(4).map(_ => v(1.1em)))
  v(6pt)
  [*Operario cualificado que ha realizado las operaciones* \ Nombre: #hueco(w: 6cm) #h(1fr) Firma: #hueco(w: 4.5cm)]
}
