// Relación de presupuestos por cliente e instalación: qué se ha ofertado y en qué estado está.
// No muestra costes internos (se puede entregar al cliente).
#import "comun.typ": *

// Entradas: datos = {titulo, fecha, lista: [ResumenPresupuesto]}, empresa, membrete.
#let empresa = json(bytes(sys.inputs.empresa))
#let membrete = json(bytes(sys.inputs.membrete))
#let d = json(bytes(sys.inputs.datos))
#let estados = (borrador: "Borrador", enviado: "Enviado", aceptado: "Aceptado", rechazado: "Rechazado")
#let tipos = (correctivo: "Reparación", obra: "Obra")
#let fecha(s) = s.slice(8, 10) + "/" + s.slice(5, 7) + "/" + s.slice(0, 4)
#let razon = empresa.at("razon_social", default: "")

#set document(title: d.titulo)
#set text(font: "Inter", size: 8.5pt, lang: "es", region: "es")
#set table(stroke: (x, y) => (bottom: 0.5pt + linea), inset: (x: 4pt, y: 3.5pt))
#show table.cell.where(y: 0): set text(weight: "bold", size: 7.5pt, fill: azul)
#set page(paper: "a4", flipped: true, margin: (x: 1.5cm, top: 1.4cm, bottom: 1.4cm),
  footer: context {
    set text(7pt, fill: gris)
    [#razon #h(1fr) #d.titulo · #fecha(d.fecha) · página #counter(page).display() de #counter(page).final().first()]
  })

#grid(columns: (auto, 1fr), column-gutter: 12pt, align: horizon,
  if membrete.logo != none { image(membrete.logo, height: 1.1cm, width: auto) },
  [#text(15pt, weight: "bold", fill: azul, d.titulo) \ #text(8pt, fill: gris)[#razon · #fecha(d.fecha) · importes sin IVA]],
)
#v(0.4em)

// Resumen por estado
#let suma(l) = l.fold(0, (a, r) => a + r.base)
#grid(columns: (1fr,) * 4, column-gutter: 6pt,
  ..estados.pairs().map(((k, t)) => {
    let l = d.lista.filter(r => r.estado == k)
    tarjeta(t + " (" + str(l.len()) + ")", eur(suma(l)), destacado: k == "enviado")
  }),
)

// Agrupar: cliente → instalación
#let clientes = d.lista.map(r => r.cliente).dedup()
#for c in clientes {
  let del_cliente = d.lista.filter(r => r.cliente == c)
  block(above: 1.2em, below: 0.4em, sticky: true,
    grid(columns: (1fr, auto), text(10.5pt, weight: "bold", fill: azul, c), text(fill: gris, eur(suma(del_cliente)))))
  for ins in del_cliente.map(r => r.instalacion).dedup() {
    let filas = del_cliente.filter(r => r.instalacion == ins)
    block(above: 0.5em, below: 0.2em, sticky: true,
      text(weight: "bold")[#if ins == "" [Sin instalación indicada] else [#ins]])
    table(columns: (2.2cm, 1.9cm, 1.8cm, 1fr, 1.9cm, 2.3cm), align: (left, left, left, left, left, right),
      table.header[Número][Fecha][Tipo][Asunto / equipo][Estado][Base],
      ..filas.map(r => (
        text(weight: "bold", r.numero), fecha(r.fecha), tipos.at(r.tipo, default: r.tipo),
        [#r.titulo #if r.equipo != "" [\ #text(7.5pt, fill: gris, r.equipo)]],
        estados.at(r.estado, default: r.estado), eur(r.base),
      )).flatten(),
    )
  }
}
