// Estilo y utilidades comunes de los informes de Mantenix.

#let azul = rgb("#1f3a5f")
#let gris = luma(110)
#let linea = luma(200)
#let fondo = luma(245)

// Número con formato español: 1.234,56
#let num(x, d: 2) = {
  if x == none { return "—" }
  let neg = x < 0
  let f = calc.pow(10, d)
  let v = int(calc.round(calc.abs(x) * f))
  let ent = str(calc.quo(v, f))
  let dec = str(calc.rem(v, f))
  while dec.len() < d { dec = "0" + dec }
  let grupos = ()
  while ent.len() > 3 {
    grupos.insert(0, ent.slice(ent.len() - 3))
    ent = ent.slice(0, ent.len() - 3)
  }
  grupos.insert(0, ent)
  (if neg { "−" } else { "" }) + grupos.join(".") + (if d > 0 { "," + dec } else { "" })
}
#let eur(x) = if x == none { "—" } else { num(x) + " €" }

#let estilo(titulo, subtitulo: none, empresa: "", doc) = {
  set document(title: titulo)
  set text(font: "Inter", size: 9pt, lang: "es", region: "es")
  set par(justify: false, leading: 0.6em)
  set table(stroke: (x, y) => (bottom: 0.5pt + linea), inset: (x: 5pt, y: 4pt))
  show table.cell.where(y: 0): set text(weight: "bold", size: 8pt, fill: azul)
  show heading.where(level: 1): it => block(above: 1.4em, below: 0.7em, sticky: true,
    text(10.5pt, weight: "bold", fill: azul, it.body))
  set page(paper: "a4", margin: (x: 1.8cm, top: 2.2cm, bottom: 2cm),
    header: context {
      set text(7.5pt, fill: gris)
      [#empresa · #titulo #h(1fr) Uso interno]
      v(-4pt); line(length: 100%, stroke: 0.5pt + linea)
    },
    footer: context {
      set text(7.5pt, fill: gris)
      [Generado con Mantenix #h(1fr) Página #counter(page).display() de #counter(page).final().first()]
    })
  text(16pt, weight: "bold", fill: azul, titulo)
  if subtitulo != none { linebreak(); text(10pt, fill: gris, subtitulo) }
  v(0.6em)
  doc
}

#let tarjeta(etiqueta, valor, destacado: false) = box(
  width: 100%, inset: 8pt, radius: 3pt,
  fill: if destacado { azul } else { fondo },
  {
    set text(fill: if destacado { white } else { black })
    text(7.5pt, fill: if destacado { white.darken(15%) } else { gris }, upper(etiqueta))
    linebreak()
    text(13pt, weight: "bold", valor)
  },
)
